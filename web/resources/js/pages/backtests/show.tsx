import { Head } from '@inertiajs/react';
import AppLayout from '@/layouts/app-layout';
import type { BreadcrumbItem } from '@/types';
import type { BacktestResult } from '@/types/models';
import { index, show } from '@/actions/App/Http/Controllers/BacktestController';

export default function BacktestsShow({ result }: { result: BacktestResult }) {
    const breadcrumbs: BreadcrumbItem[] = [
        { title: 'Backtests', href: index.url() },
        { title: `#${result.id}`, href: show.url(result.id) },
    ];

    const metrics = [
        { label: 'Total Trades', value: result.total_trades ?? '-' },
        {
            label: 'Win Rate',
            value: result.win_rate
                ? `${(parseFloat(result.win_rate) * 100).toFixed(1)}%`
                : '-',
        },
        {
            label: 'PnL',
            value: result.total_pnl_usdc
                ? `$${parseFloat(result.total_pnl_usdc).toFixed(2)}`
                : '-',
        },
        {
            label: 'Max Drawdown',
            value: result.max_drawdown
                ? `${(parseFloat(result.max_drawdown) * 100).toFixed(1)}%`
                : '-',
        },
        { label: 'Sharpe Ratio', value: result.sharpe_ratio ?? '-' },
    ];

    return (
        <AppLayout breadcrumbs={breadcrumbs}>
            <Head title={`Backtest #${result.id}`} />
            <div className="p-6">
                <h1 className="mb-2 text-2xl font-bold">
                    Backtest #{result.id}
                </h1>
                <p className="mb-6 text-muted-foreground">
                    Strategy: {result.strategy.name}
                    {result.date_from && result.date_to && (
                        <>
                            {' '}
                            · {new Date(
                                result.date_from,
                            ).toLocaleDateString()}{' '}
                            – {new Date(result.date_to).toLocaleDateString()}
                        </>
                    )}
                </p>
                <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-5">
                    {metrics.map((m) => (
                        <div
                            key={m.label}
                            className="rounded-xl border border-sidebar-border/70 p-4 dark:border-sidebar-border"
                        >
                            <p className="text-sm text-muted-foreground">
                                {m.label}
                            </p>
                            <p className="text-2xl font-bold">{m.value}</p>
                        </div>
                    ))}
                </div>
            </div>
        </AppLayout>
    );
}
