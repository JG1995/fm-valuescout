/**
 * OMP Custom Command: /project-info
 *
 * A slash command that displays git repository information.
 * Runs `git remote -v` and `git branch --show-current`, then returns
 * the output as a formatted prompt for the LLM to summarize.
 *
 * Usage in OMP chat: /project-info
 */
export default function createProjectInfoCommand(api) {
  const { cwd, exec } = api;

  return {
    name: 'project-info',
    description: 'Show git repository information (remote and current branch)',
    execute: async function(args, ctx) {
      try {
        // Run git commands to gather repository info
        const remoteResult = await exec('git', ['remote', '-v'], { cwd });
        const branchResult = await exec('git', ['branch', '--show-current'], { cwd });

        // Format the output as a prompt for the LLM
        return `## Git Repository Information

**Remote repositories:**
\`\`\`
${remoteResult.stdout.trim() || '(no remotes configured)'}
\`\`\`

**Current branch:**
\`\`\`
${branchResult.stdout.trim() || '(unable to determine)'}
\`\`\`

Please summarize this repository's remote configuration and current branch status.`;

      } catch (error) {
        // Handle non-git repos or git command failures gracefully
        return `## Git Repository Information

Unable to retrieve git information. This may not be a git repository, or git may not be installed.

Error: ${error.message}`;
      }
    }
  };
}
