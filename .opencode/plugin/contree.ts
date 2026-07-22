import { existsSync, readFileSync } from "node:fs";
import { join, basename } from "node:path";
import {
  endsWithQuestion,
  validateMentalModel,
  buildDriftNudges,
  shouldSelfCareNudge,
  SELF_CARE_NUDGE,
} from "../lib/contree-core.ts";

const HEARTBEAT_PRUNE_AGE = 3600;

function nowSeconds(): number {
  const override = process.env.CONTREE_NOW;
  return override ? Number(override) : Math.floor(Date.now() / 1000);
}

export const Contree = async ({ directory, client }: any) => {
  const lastTextBySession = new Map<string, string>();
  const drivenThisTurn = new Set<string>();
  const selfInjected = new Set<string>();
  const heartbeats: number[] = [];
  let lastNudge: number | null = null;

  return {
    "chat.message": async (input: any, output: any) => {
      // Our own re-drive prompt also arrives as a message — consume the flag and
      // do NOT rearm or record activity, or idle→prompt→idle loops forever.
      if (selfInjected.has(input.sessionID)) {
        selfInjected.delete(input.sessionID);
        return;
      }
      drivenThisTurn.delete(input.sessionID); // real user message → rearm the drift guard

      // Self-care: a real user message is keyboard activity. After 20 min of
      // continuous activity, prepend a 20-20-20 eye-break reminder (throttled).
      const now = nowSeconds();
      heartbeats.push(now);
      const recent = heartbeats.filter((t) => now - t <= HEARTBEAT_PRUNE_AGE);
      heartbeats.length = 0;
      heartbeats.push(...recent);
      if (shouldSelfCareNudge(heartbeats, now, lastNudge)) {
        lastNudge = now;
        if (Array.isArray(output?.parts)) output.parts.unshift({ type: "text", text: SELF_CARE_NUDGE });
      }
    },

    "experimental.text.complete": async (input: any, output: any) => {
      if (output?.text) lastTextBySession.set(input.sessionID, output.text);
    },

    "tool.execute.after": async (input: any, output: any) => {
      const fp = input?.args?.filePath;
      const isMentalModelEdit =
        (input.tool === "edit" || input.tool === "write") &&
        typeof fp === "string" &&
        basename(fp) === "MENTAL_MODEL.md";
      if (!isMentalModelEdit) return;
      const mmPath = join(directory, "MENTAL_MODEL.md");
      const content = existsSync(mmPath) ? readFileSync(mmPath, "utf-8") : null;
      const findings = validateMentalModel(content);
      if (findings.length && output) {
        output.output = `${output.output ?? ""}\n\nMENTAL_MODEL.md validator findings:\n${findings.join("\n")}`;
      }
    },

    event: async ({ event }: any) => {
      if (event.type !== "session.idle") return;
      const sessionID = event.properties?.sessionID ?? event.properties?.info?.id;
      if (!sessionID || drivenThisTurn.has(sessionID)) return;
      if (endsWithQuestion(lastTextBySession.get(sessionID) ?? "")) return; // yield to the user
      drivenThisTurn.add(sessionID);
      selfInjected.add(sessionID);
      const nudges = buildDriftNudges(
        existsSync(join(directory, "MENTAL_MODEL.md")),
        existsSync(join(directory, "README.md")),
      );
      await client.session
        .prompt({ path: { id: sessionID }, body: { parts: [{ type: "text", text: nudges }] } })
        .catch(() => {});
    },
  };
};
