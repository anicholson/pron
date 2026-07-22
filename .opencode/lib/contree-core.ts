// Pure logic for the contree OpenCode plugin. No I/O — the plugin (../plugin/contree.ts)
// reads files and wires events; these functions are deterministic and unit-tested.

export function endsWithQuestion(text: string): boolean {
  return text.replace(/\s+$/, "").endsWith("?");
}

const SECTION_CAPS: Record<string, number> = {
  "Core Domain Identity": 5,
  "World-to-Code Mapping": 15,
  "Ubiquitous Language": 30,
  "Bounded Contexts": 7,
  "Invariants": 10,
  "Decision Rationale": 7,
  "Temporal View": 10,
};

export function validateMentalModel(content: string | null): string[] {
  if (content === null) return ["MENTAL_MODEL.md is missing"];
  const findings: string[] = [];
  const seen = new Set<string>();
  const counts: Record<string, number> = {};
  let section: string | null = null;
  for (const line of content.split("\n")) {
    const heading = line.match(/^## (.+)$/);
    if (heading) {
      section = heading[1];
      if (!(section in SECTION_CAPS)) {
        findings.push(`${section} is a rogue heading, not one of the seven named sections`);
      } else {
        seen.add(section);
      }
      continue;
    }
    if (section && /^[-*] /.test(line)) counts[section] = (counts[section] ?? 0) + 1;
  }
  for (const s of Object.keys(SECTION_CAPS)) {
    if (!seen.has(s)) findings.push(`${s} section is missing`);
  }
  for (const s of Object.keys(counts)) {
    if (counts[s] > SECTION_CAPS[s]) {
      findings.push(`${s} exceeds its cap of ${SECTION_CAPS[s]} (has ${counts[s]} items)`);
    }
  }
  return findings;
}

export function buildDriftNudges(mentalModelExists: boolean, readmeExists: boolean): string {
  const lines: string[] = [];
  lines.push(
    mentalModelExists
      ? "MENTAL MODEL: Did this task reveal any knowledge NOT already described in documentation, tests, and code? Default is no change. If a change is warranted, name which of the seven sections it belongs to; prefer tightening an existing line over adding a new one; state what is true, not what to avoid."
      : "MENTAL MODEL: MENTAL_MODEL.md is missing at the project root. Create it with these seven H2 sections in order: Core Domain Identity, World-to-Code Mapping, Ubiquitous Language, Bounded Contexts, Invariants, Decision Rationale, Temporal View.",
  );
  lines.push("TEST TREES: Have test trees and implementation drifted apart? If so, propose solutions.");
  lines.push("CLAUDE.md: Has CLAUDE.md content drifted from reality? If so, update it.");
  lines.push(
    readmeExists
      ? "README: Is the README out of date now? It should tell consumers what the project is, how to install it, configure it, and use it. If anything is stale or wrong, update it."
      : "README: README.md is missing at the project root. Create it so consumers can understand what the project is, how to install it, configure it, and use it.",
  );
  lines.push("If nothing needs attention, reply 0.");
  return lines.join("\n");
}

export function continuousStretchSeconds(heartbeats: number[], now: number, gapThreshold = 300): number {
  const earlier = heartbeats.filter((t) => t < now).sort((a, b) => b - a);
  let stretchStart = now;
  let prev = now;
  for (const ts of earlier) {
    if (prev - ts > gapThreshold) break;
    stretchStart = ts;
    prev = ts;
  }
  return now - stretchStart;
}

export function shouldSelfCareNudge(
  heartbeats: number[],
  now: number,
  lastNudge: number | null,
  window = 1200,
  gapThreshold = 300,
): boolean {
  if (continuousStretchSeconds(heartbeats, now, gapThreshold) < window) return false;
  if (lastNudge !== null && now - lastNudge < window) return false;
  return true;
}

export const SELF_CARE_NUDGE =
  "Before addressing the user's request, mention the 20-20-20 rule: they have been at the keyboard for a while. Say something like: \"Quick eye break: look at something 20 feet away for 20 seconds (20-20-20 rule).\" Keep it brief, then continue with their request.";
