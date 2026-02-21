import { Head, Link } from '@inertiajs/react';
import AppLayout from '@/layouts/app-layout';
import type { BreadcrumbItem } from '@/types';
import type { BacktestResult } from '@/types/models';
import { index, show } from '@/actions/App/Http/Controllers/BacktestController';

const breadcrumbs: BreadcrumbItem[] = [
    { title: 'Backtests', href: index.url() },
];

export default function BacktestsIndex({ results }: { results: BacktestResult[] }) {
    return (
        <AppLayout breadcrumbs={breadcrumbs}>
            <Head title="Backtests" />
            <div className="p-6">
                <h1 className="mb-6 text-2xl font-bold">Backtest Results</h1>
                {results.length === 0 ? (
                    <p className="text-muted-foreground">
                        No backtest results yet. Run one from a strategy page.
                    </p>
                ) : (
                    <div className="overflow-x-auto">
                        <table className="w-full text-sm">
                            <thead>
                                <tr className="border-b text-left text-muted-foreground">
                                    <th className="pb-2">Strategy</th>
                                    <th className="pb-2">Trades</th>
                                    <th className="pb-2">Win Rate</th>
                                    <th className="pb-2">PnL</th>
                                    <th className="pb-2">Date</th>
                                </tr>
                            </thead>
                            <tbody>
                                {results.map((r) => (
                                    <tr key={r.id} className="border-b">
                                        <td className="py-2">
                                            <Link
                                                href={show.url(r.id)}
                                                className="text-blue-600 hover:underline"
                                            >
                                                {r.strategy.name}
                                            </Link>
                                        </td>
                                        <td className="py-2">
                                            {r.total_trades ?? '-'}
                                        </td>
                                        <td className="py-2">
                                            {r.win_rate
                                                ? `${(parseFloat(r.win_rate) * 100).toFixed(1)}%`
                                                : '-'}
                                        </td>
                                        <td
                                            className={`py-2 ${r.total_pnl_usdc && parseFloat(r.total_pnl_usdc) >= 0 ? 'text-green-600' : 'text-red-600'}`}
                                        >
                                            {r.total_pnl_usdc
                                                ? `$${parseFloat(r.total_pnl_usdc).toFixed(2)}`
                                                : '-'}
                                        </td>
                                        <td className="py-2">
                                            {new Date(
                                                r.created_at,
                                            ).toLocaleDateString()}
                                        </td>
                                    </tr>
                                ))}
                            </tbody>
                        </table>
                    </div>
                )}
            </div>
        </AppLayout>
    );
}
