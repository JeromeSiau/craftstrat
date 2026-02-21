import { Head, router } from '@inertiajs/react';
import AppLayout from '@/layouts/app-layout';
import { Button } from '@/components/ui/button';
import type { BreadcrumbItem } from '@/types';
import type { Strategy, WalletStrategy, BacktestResult } from '@/types/models';
import { index, show, activate, deactivate, destroy } from '@/actions/App/Http/Controllers/StrategyController';

interface StrategyShowProps {
    id: number;
    name: string;
    description: string | null;
    mode: string;
    graph: Record<string, unknown>;
    is_active: boolean;
    wallet_strategies: WalletStrategy[];
    backtest_results: BacktestResult[];
}

export default function StrategiesShow({ strategy }: { strategy: StrategyShowProps }) {
    const breadcrumbs: BreadcrumbItem[] = [
        { title: 'Strategies', href: index.url() },
        { title: strategy.name, href: show.url(strategy.id) },
    ];

    return (
        <AppLayout breadcrumbs={breadcrumbs}>
            <Head title={strategy.name} />
            <div className="p-6">
                <div className="mb-6 flex items-center justify-between">
                    <div>
                        <h1 className="text-2xl font-bold">{strategy.name}</h1>
                        {strategy.description && (
                            <p className="mt-1 text-muted-foreground">
                                {strategy.description}
                            </p>
                        )}
                    </div>
                    <div className="flex gap-2">
                        {strategy.is_active ? (
                            <Button
                                variant="outline"
                                onClick={() =>
                                    router.post(
                                        deactivate.url(strategy.id),
                                    )
                                }
                            >
                                Deactivate
                            </Button>
                        ) : (
                            <Button
                                onClick={() =>
                                    router.post(
                                        activate.url(strategy.id),
                                    )
                                }
                            >
                                Activate
                            </Button>
                        )}
                        <Button
                            variant="destructive"
                            onClick={() => {
                                if (confirm('Are you sure you want to delete this strategy? This action cannot be undone.')) {
                                    router.delete(destroy.url(strategy.id));
                                }
                            }}
                        >
                            Delete
                        </Button>
                    </div>
                </div>

                <div className="grid gap-6 md:grid-cols-2">
                    <div className="rounded-lg border border-sidebar-border p-4">
                        <h2 className="mb-3 font-semibold">Configuration</h2>
                        <dl className="space-y-2 text-sm">
                            <div className="flex justify-between">
                                <dt className="text-muted-foreground">Mode</dt>
                                <dd>{strategy.mode}</dd>
                            </div>
                            <div className="flex justify-between">
                                <dt className="text-muted-foreground">
                                    Status
                                </dt>
                                <dd>
                                    {strategy.is_active ? 'Active' : 'Inactive'}
                                </dd>
                            </div>
                        </dl>
                    </div>

                    <div className="rounded-lg border border-sidebar-border p-4">
                        <h2 className="mb-3 font-semibold">Assigned Wallets</h2>
                        {strategy.wallet_strategies.length === 0 ? (
                            <p className="text-sm text-muted-foreground">
                                No wallets assigned.
                            </p>
                        ) : (
                            <ul className="space-y-2 text-sm">
                                {strategy.wallet_strategies.map((ws) => (
                                    <li
                                        key={ws.id}
                                        className="flex items-center justify-between"
                                    >
                                        <span className="font-mono text-xs">
                                            {ws.wallet.label ||
                                                ws.wallet.address.slice(0, 10) +
                                                    '...'}
                                        </span>
                                        <span
                                            className={
                                                ws.is_running
                                                    ? 'text-green-600'
                                                    : 'text-gray-400'
                                            }
                                        >
                                            {ws.is_running
                                                ? 'Running'
                                                : 'Stopped'}
                                        </span>
                                    </li>
                                ))}
                            </ul>
                        )}
                    </div>
                </div>

                <div className="mt-6 rounded-lg border border-sidebar-border p-4">
                    <h2 className="mb-3 font-semibold">Recent Backtests</h2>
                    {strategy.backtest_results.length === 0 ? (
                        <p className="text-sm text-muted-foreground">
                            No backtests yet.
                        </p>
                    ) : (
                        <div className="overflow-x-auto">
                            <table className="w-full text-sm">
                                <thead>
                                    <tr className="border-b text-left text-muted-foreground">
                                        <th className="pb-2">Date</th>
                                        <th className="pb-2">Trades</th>
                                        <th className="pb-2">Win Rate</th>
                                        <th className="pb-2">PnL</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {strategy.backtest_results.map((bt) => (
                                        <tr key={bt.id} className="border-b">
                                            <td className="py-2">
                                                {new Date(
                                                    bt.created_at,
                                                ).toLocaleDateString()}
                                            </td>
                                            <td className="py-2">
                                                {bt.total_trades}
                                            </td>
                                            <td className="py-2">
                                                {bt.win_rate
                                                    ? `${(parseFloat(bt.win_rate) * 100).toFixed(1)}%`
                                                    : '-'}
                                            </td>
                                            <td
                                                className={`py-2 ${bt.total_pnl_usdc && parseFloat(bt.total_pnl_usdc) >= 0 ? 'text-green-600' : 'text-red-600'}`}
                                            >
                                                {bt.total_pnl_usdc
                                                    ? `$${parseFloat(bt.total_pnl_usdc).toFixed(2)}`
                                                    : '-'}
                                            </td>
                                        </tr>
                                    ))}
                                </tbody>
                            </table>
                        </div>
                    )}
                </div>
            </div>
        </AppLayout>
    );
}
