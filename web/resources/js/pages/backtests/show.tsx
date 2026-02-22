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

    const pnlValue = parseFloat(result.total_pnl_usdc || '0');
    const drawdownValue = parseFloat(result.max_drawdown || '0');

    const metrics = [
        { label: 'Total Trades', value: result.total_trades ?? '-' },
        { label: 'Win Rate', value: formatWinRate(result.win_rate) },
        { label: 'PnL', value: formatPnl(result.total_pnl_usdc), trend: (pnlValue > 0 ? 'up' : pnlValue < 0 ? 'down' : 'neutral') as 'up' | 'down' | 'neutral' },
        { label: 'Max Drawdown', value: formatPercentage(result.max_drawdown), trend: (drawdownValue < 0 ? 'down' : 'neutral') as 'up' | 'down' | 'neutral' },
        { label: 'Sharpe Ratio', value: result.sharpe_ratio ?? '-' },
    ];

    return (
        <AppLayout breadcrumbs={breadcrumbs}>
            <Head title={`Backtest #${result.id}`} />
            <div className="p-4 md:p-8">
                <div className="mb-8">
                    <h1 className="text-2xl font-bold tracking-tight">
                        Backtest #{result.id}
                    </h1>
                    <p className="mt-1 text-muted-foreground">
                        Strategy: {result.strategy.name}
                        {result.date_from && result.date_to && (
                            <>
                                {' '}
                                · {new Date(result.date_from).toLocaleDateString()}{' '}
                                – {new Date(result.date_to).toLocaleDateString()}
                            </>
                        )}
                    </p>
                </div>

                <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-5">
                    {metrics.map((m) => (
                        <MetricCard key={m.label} label={m.label} value={m.value} trend={m.trend} />
                    ))}
                </div>

                {result.result_detail?.trades && result.result_detail.trades.length > 0 && (
                    <Card className="mt-6 border-l-4 border-l-emerald-500/50">
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
