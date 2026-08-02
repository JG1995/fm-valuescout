export function resolveUiAgentForwardConsole(
  env: Record<string, string | undefined>,
) {
  return env.FM_VALUESCOUT_UI_AGENT === "1" &&
    env.FM_VALUESCOUT_UI_AGENT_WSL === "1"
    ? false
    : undefined;
}
