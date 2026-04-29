/**
 * Git mutation confirmation hook
 * Blocks git add/commit commands unless explicitly approved by the user
 * @param {object} pi - Hook API
 * @param {Function} pi.on - Event listener: (eventName, handler) => void
 */
export default function (pi) {
  const BLOCKED_PATTERNS = [
    /\bgit\s+add\b/,
    /\bgit\s+commit\b/,
  ];

  pi.on("tool_call", async (event, ctx) => {
    if (event.toolName !== "bash") return;
    const cmd = String(event.input.command ?? "");

    const matched = BLOCKED_PATTERNS.find(p => p.test(cmd));
    if (!matched) return;

    if (!ctx.hasUI) {
      return { block: true, reason: `Git mutation blocked (no UI): ${cmd}` };
    }

    const approved = await ctx.ui.confirm(
      "Git Mutation Blocked",
      `The agent wants to run:\n\n${cmd}\n\nAllow this command?`
    );

    if (!approved) {
      return { block: true, reason: `User denied git mutation: ${cmd}` };
    }
  });
}
