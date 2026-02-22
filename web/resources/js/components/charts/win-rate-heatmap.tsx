import type { HeatmapCell } from '@/types/models';

interface WinRateHeatmapProps {
    data: HeatmapCell[];
}

// Move bins from the Rust multiIf, ordered top-to-bottom (positive moves first)
const MOVE_BINS = ['>= 0.2', '0.1 / 0.2', '0 / 0.1', '-0.1 / 0', '-0.2 / -0.1', '< -0.2'];

function cellColor(winRate: number, total: number): string {
    if (total < 3) return 'bg-muted text-muted-foreground';
    if (winRate >= 65) return 'bg-emerald-700 text-white';
    if (winRate >= 55) return 'bg-emerald-600 text-white';
    if (winRate >= 50) return 'bg-emerald-400 text-emerald-950';
    if (winRate >= 45) return 'bg-red-400 text-red-950';
    if (winRate >= 35) return 'bg-red-600 text-white';
    return 'bg-red-700 text-white';
}

function sortTimeBins(bins: string[]): string[] {
    return [...bins].sort((a, b) => {
        const numA = parseFloat(a.split('-')[0]);
        const numB = parseFloat(b.split('-')[0]);
        return numA - numB;
    });
}

export function WinRateHeatmap({ data }: WinRateHeatmapProps) {
    if (data.length === 0) {
        return <p className="py-8 text-center text-sm text-muted-foreground">No heatmap data available.</p>;
    }

    // Extract and sort unique time bins
    const timeBins = sortTimeBins([...new Set(data.map((d) => d.time_bin))]);

    // Build lookup map: "moveBin|timeBin" -> cell
    const lookup = new Map<string, HeatmapCell>();
    for (const cell of data) {
        lookup.set(`${cell.move_bin}|${cell.time_bin}`, cell);
    }

    return (
        <div className="overflow-x-auto">
            <table className="w-full border-collapse text-xs">
                <thead>
                    <tr>
                        <th className="border border-border bg-muted px-2 py-1.5 text-left font-medium text-muted-foreground">
                            Move \ Time
                        </th>
                        {timeBins.map((tb) => (
                            <th
                                key={tb}
                                className="border border-border bg-muted px-2 py-1.5 text-center font-medium text-muted-foreground"
                            >
                                {tb}
                            </th>
                        ))}
                    </tr>
                </thead>
                <tbody>
                    {MOVE_BINS.map((mb) => (
                        <tr key={mb}>
                            <td className="border border-border bg-muted px-2 py-1.5 font-medium text-muted-foreground whitespace-nowrap">
                                {mb}
                            </td>
                            {timeBins.map((tb) => {
                                const cell = lookup.get(`${mb}|${tb}`);
                                const total = cell?.total ?? 0;
                                const winRate = cell ? cell.win_rate * 100 : 0;

                                return (
                                    <td
                                        key={tb}
                                        className={`border border-border px-2 py-1.5 text-center ${cellColor(winRate, total)}`}
                                    >
                                        {total < 3 ? (
                                            <span className="opacity-50">-</span>
                                        ) : (
                                            <>
                                                <div className="font-semibold">{winRate.toFixed(0)}%</div>
                                                <div className="text-[10px] opacity-75">n={total}</div>
                                            </>
                                        )}
                                    </td>
                                );
                            })}
                        </tr>
                    ))}
                </tbody>
            </table>
        </div>
    );
}
