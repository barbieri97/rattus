// Tab-separated text, for copying data to spreadsheets.

type Cell = string | number | boolean | null | undefined;

function cell(value: Cell): string {
  if (value === null || value === undefined) return "";
  if (typeof value === "number") {
    return Number.isFinite(value) ? String(Math.round(value * 10000) / 10000) : "";
  }
  return String(value).replace(/[\t\r\n]+/g, " ");
}

export function toTsv(header: string[], rows: Cell[][]): string {
  return [header, ...rows].map((r) => r.map(cell).join("\t")).join("\n") + "\n";
}
