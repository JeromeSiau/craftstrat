import { Link } from '@inertiajs/react';
import { formatWinRate, formatPnl, pnlColorClass } from '@/lib/formatters';
import type { BacktestResult } from '@/types/models';

interface BacktestResultsTableProps {
    results: BacktestResult[];
    showStrategy?: boolean;
    linkBuilder?: (result: BacktestResult) => string;
}

export default function BacktestResultsTable({
    results,
    showStrategy = false,
    linkBuilder,
}: BacktestResultsTableProps) {
    return (
        <div className="overflow-x-auto">
            <table className="w-full text-sm">
                <thead>
                    <tr className="border-b text-left text-muted-foreground">
                        {showStrategy && <th className="pb-2">Strategy</th>}
                        <th className="pb-2">Trades</th>
                        <th className="pb-2">Win Rate</th>
                        <th className="pb-2">PnL</th>
                        <th className="pb-2">Date</th>
                    </tr>
                </thead>
                <tbody>
                    {results.map((r) => (
                        <tr key={r.id} className="border-b">
                            {showStrategy && (
                                <td className="py-2">
                                    {linkBuilder ? (
                                        <Link
                                            href={linkBuilder(r)}
                                            className="text-blue-600 hover:underline"
                                        >
                                            {r.strategy.name}
                                        </Link>
                                    ) : (
                                        r.strategy.name
                                    )}
                                </td>
                            )}
                            <td className="py-2">{r.total_trades ?? '-'}</td>
                            <td className="py-2">{formatWinRate(r.win_rate)}</td>
                            <td className={`py-2 ${pnlColorClass(r.total_pnl_usdc)}`}>
                                {formatPnl(r.total_pnl_usdc)}
                            </td>
                            <td className="py-2">
                                {new Date(r.created_at).toLocaleDateString()}
                            </td>
                        </tr>
                    ))}
                </tbody>
            </table>
        </div>
    );
}
