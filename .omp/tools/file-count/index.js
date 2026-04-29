/**
 * OMP Custom Tool: file-count
 *
 * A custom tool that counts files matching a glob pattern in the project.
 * Demonstrates the full custom tool contract using the injected TypeBox API.
 *
 * Usage in OMP: LLM can call this tool to count files matching a pattern
 */
export default function createFileCountTool(api) {
  const { cwd, exec, typebox } = api;
  const { Type } = typebox;

  return {
    name: 'file-count',
    label: 'File Counter',
    description: 'Count files in the project matching a glob pattern',
    parameters: Type.Object({
      pattern: Type.String({
        description: 'Glob pattern to match files (e.g., "src/**/*.ts", "*.md")',
        examples: ['src/**/*.ts', '*.md', '**/*.json']
      }),
      max_depth: Type.Optional(Type.Integer({
        description: 'Maximum directory depth to search',
        default: 10
      }))
    }),
    execute: async function(toolCallId, params, onUpdate, ctx, signal) {
      const { pattern, max_depth = 10 } = params;

      try {
        // Use find command to count files matching the pattern
        // We'll use a simple approach with find and head for depth limiting
        const { stdout } = await exec('find', [cwd, '-type', 'f', '-name', pattern], {
          cwd,
          signal
        });

        const files = stdout.trim().split('\n').filter(f => f);
        const count = files.length;

        // Return result in AgentToolResult format
        return {
          content: [
            {
              type: 'text',
              text: `Found ${count} file(s) matching pattern "${pattern}"`
            }
          ],
          details: {
            pattern,
            count,
            files: files.slice(0, 20) // Include first 20 files as preview
          }
        };

      } catch (error) {
        // Handle errors gracefully
        return {
          content: [
            {
              type: 'text',
              text: `Error counting files: ${error.message}`
            }
          ],
          details: {
            pattern,
            error: error.message
          }
        };
      }
    }
  };
}
