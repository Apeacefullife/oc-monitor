export interface ReportSection {
  title: string;
  items: string[];
}

const SECTION_HEAD =
  /^【([^】]+)】$|^##\s+(.+)$|^\*\*(.+)\*\*$|^【([^】]+)】(.+)$/;

function stripItemPrefix(line: string): string {
  return line
    .replace(/^[·•\-]\s+/, "")
    .replace(/^\d+[.)]\s+/, "")
    .trim();
}

export function parseStructuredReport(text: string): ReportSection[] {
  const sections: ReportSection[] = [];
  let current: ReportSection | null = null;

  for (const rawLine of text.split("\n")) {
    const line = rawLine.trim();
    if (!line) continue;

    const sectionMatch = line.match(SECTION_HEAD);
    if (sectionMatch) {
      const title =
        sectionMatch[1] ??
        sectionMatch[2] ??
        sectionMatch[3] ??
        sectionMatch[4] ??
        "";
      const inline = sectionMatch[5]?.trim();
      current = { title: title.trim(), items: inline ? [inline] : [] };
      sections.push(current);
      continue;
    }

    if (!current) {
      current = { title: "", items: [] };
      sections.push(current);
    }
    current.items.push(stripItemPrefix(line));
  }

  return sections.filter((s) => s.title || s.items.length > 0);
}

export function insightsToSections(
  insights: string[],
  sectionTitle: string,
): ReportSection[] {
  if (insights.length === 0) return [];
  return [{ title: sectionTitle, items: insights }];
}

export function mergeReportSections(
  sections: ReportSection[],
): ReportSection[] {
  return sections.filter((s) => s.items.some((item) => item.trim().length > 0));
}
