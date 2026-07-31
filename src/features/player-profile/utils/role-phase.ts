/** Map IPC/DB phase strings to the short IP/OOP labels used in the Roles UI. */
export function rolePhaseLabel(phase: string): string {
  if (phase === "ip" || phase === "in_possession") return "IP";
  if (phase === "oop" || phase === "out_of_possession") return "OOP";
  return phase.toUpperCase();
}
