import { Head } from '@inertiajs/react';
import AppLayout from '@/layouts/app-layout';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { PnlChart } from '@/components/charts/pnl-chart';
import MetricCard from '@/components/metric-card';
import { formatWinRate, formatPnl, formatPercentage } from '@/lib/formatters';
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
        { label: 'Win Rate', value: formatWinRate(result.win_rate) },
        { label: 'PnL', value: formatPnl(result.total_pnl_usdc) },
        { label: 'Max Drawdown', value: formatPercentage(result.max_drawdown) },
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
                            · {new Date(result.date_from).toLocaleDateString()}{' '}
                            – {new Date(result.date_to).toLocaleDateString()}
                        </>
                    )}
                </p>
                <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-5">
                    {metrics.map((m) => (
                        <MetricCard key={m.label} label={m.label} value={m.value} />
                    ))}
                </div>

                {result.result_detail?.trades && result.result_detail.trades.length > 0 && (
                    <Card className="mt-6">
                        <CardHeader>
                            <CardTitle>Cumulative PnL</CardTitle>
                        </CardHeader>
                        <CardContent>
                            <PnlChart trades={result.result_detail.trades} />
                        </CardContent>
                    </Card>
                )}
            </div>
        </AppLayout>
    );
}
