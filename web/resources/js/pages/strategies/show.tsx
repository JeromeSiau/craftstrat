import { Deferred, Head, router, useForm } from '@inertiajs/react';
import { Activity, ArrowLeftRight, FlaskConical, LineChart, OctagonX, Settings2, TrendingUp, Wallet } from 'lucide-react';
import { run as runBacktest } from '@/actions/App/Http/Controllers/BacktestController';
import { index, show, activate, deactivate, destroy, kill, unkill } from '@/actions/App/Http/Controllers/StrategyController';
import BacktestResultsTable from '@/components/backtest-results-table';
import ConfirmDialog from '@/components/confirm-dialog';
import InputError from '@/components/input-error';
import MetricCard from '@/components/metric-card';
import StatusBadge from '@/components/status-badge';
import StrategyRulesDisplay, { isFormModeGraph } from '@/components/strategy/strategy-rules-display';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Skeleton } from '@/components/ui/skeleton';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import AppLayout from '@/layouts/app-layout';
import { formatPnl, formatWinRate } from '@/lib/formatters';
import type { BreadcrumbItem } from '@/types';
import type { LiveStats, Strategy, Trade } from '@/types/models';

function LiveDataSkeleton() {
    return (
        <>
            <div className="mt-6 grid gap-4 sm:grid-cols-3">
                {Array.from({ length: 3 }).map((_, i) => (
                    <Card key={i} className="relative overflow-hidden">
                        <CardContent className="pt-5 pb-5">
                            <div className="space-y-2">
                                <Skeleton className="h-3 w-24" />
                                <Skeleton className="h-8 w-20" />
                            </div>
                        </CardContent>
                    </Card>
                ))}
            </div>
            <Card className="mt-6">
                <CardHeader>
                    <Skeleton className="h-5 w-32" />
                </CardHeader>
                <CardContent>
                    <div className="space-y-3">
                        {Array.from({ length: 5 }).map((_, i) => (
                            <Skeleton key={i} className="h-4 w-full" />
                        ))}
                    </div>
                </CardContent>
            </Card>
        </>
    );
}

interface Props {
    strategy: Strategy;
    liveStats?: LiveStats;
    recentTrades?: Trade[];
}

export default function StrategiesShow({ strategy, liveStats, recentTrades }: Props) {
    const breadcrumbs: BreadcrumbItem[] = [
        { title: 'Strategies', href: index.url() },
        { title: strategy.name, href: show.url(strategy.id) },
    ];

    const backtestForm = useForm({
        date_from: '',
        date_to: '',
        market_filter: [] as string[],
    });

    function handleBacktestSubmit(e: React.FormEvent): void {
        e.preventDefault();
        backtestForm.post(runBacktest.url(strategy.id));
    }

    const currentStats = liveStats?.live;
    const paperStats = liveStats?.paper;
    const pnlValue = currentStats?.total_pnl_usdc ? parseFloat(currentStats.total_pnl_usdc) : 0;
    const pnlTrend = pnlValue > 0 ? 'up' : pnlValue < 0 ? 'down' : 'neutral' as const;
    const paperPnlValue = paperStats?.total_pnl_usdc ? parseFloat(paperStats.total_pnl_usdc) : 0;
    const paperPnlTrend = paperPnlValue > 0 ? 'up' : paperPnlValue < 0 ? 'down' : 'neutral' as const;

    return (
        <AppLayout breadcrumbs={breadcrumbs}>
            <Head title={strategy.name} />
            <div className="p-4 md:p-8">
                <div className="mb-8 flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
                    <div>
                        <div className="flex items-center gap-3">
                            <h1 className="text-2xl font-bold tracking-tight">{strategy.name}</h1>
                            <StatusBadge active={strategy.is_active} />
                        </div>
                        {strategy.description && (
                            <p className="mt-1 text-muted-foreground">
                                {strategy.description}
                            </p>
                        )}
                    </div>
                    <div className="flex gap-2">
                        {strategy.is_active && (
                            <>
                                <ConfirmDialog
                                    trigger={
                                        <Button variant="outline" className="border-red-500/50 text-red-600 hover:bg-red-50 dark:text-red-400 dark:hover:bg-red-950">
                                            <OctagonX className="mr-1.5 size-4" />
                                            Kill
                                        </Button>
                                    }
                                    title="Activate Kill Switch"
                                    description="This will immediately stop all evaluation for this strategy across all wallets. No new trades will be placed. Use Resume to restart."
                                    confirmLabel="Kill"
                                    onConfirm={() => router.post(kill.url(strategy.id))}
                                />
                                <Button
                                    variant="outline"
                                    onClick={() => router.post(unkill.url(strategy.id))}
                                >
                                    Resume
                                </Button>
                            </>
                        )}
                        {strategy.is_active ? (
                            <Button
                                variant="outline"
                                onClick={() => router.post(deactivate.url(strategy.id))}
                            >
                                Deactivate
                            </Button>
                        ) : (
                            <Button onClick={() => router.post(activate.url(strategy.id))}>
                                Activate
                            </Button>
                        )}
                        <ConfirmDialog
                            trigger={<Button variant="destructive">Delete</Button>}
                            title="Delete Strategy"
                            description="Are you sure you want to delete this strategy? This action cannot be undone."
                            confirmLabel="Delete"
                            onConfirm={() => router.delete(destroy.url(strategy.id))}
                        />
                    </div>
                </div>

                <div className="grid gap-6 lg:grid-cols-5">
                    <Card className="border-l-4 border-l-blue-500/50 lg:col-span-3">
                        <CardHeader>
                            <div className="flex items-center gap-3">
                                <div className="rounded-lg bg-blue-500/10 p-2 dark:bg-blue-500/15">
                                    <Settings2 className="size-4 text-blue-600 dark:text-blue-400" />
                                </div>
                                <CardTitle>Configuration</CardTitle>
                            </div>
                        </CardHeader>
                        <CardContent>
                            <dl className="grid gap-4 sm:grid-cols-2">
                                <div className="rounded-lg bg-muted/50 p-3">
                                    <dt className="text-xs font-medium tracking-wide text-muted-foreground uppercase">Mode</dt>
                                    <dd className="mt-1 font-semibold capitalize">{strategy.mode}</dd>
                                </div>
                                <div className="rounded-lg bg-muted/50 p-3">
                                    <dt className="text-xs font-medium tracking-wide text-muted-foreground uppercase">Status</dt>
                                    <dd className="mt-1 font-semibold">{strategy.is_active ? 'Active' : 'Inactive'}</dd>
                                </div>
                            </dl>

                            {strategy.graph && isFormModeGraph(strategy.graph as Record<string, unknown>) && (
                                <div className="mt-6">
                                    <StrategyRulesDisplay graph={strategy.graph} />
                                </div>
                            )}
                        </CardContent>
                    </Card>

                    <Card className="border-l-4 border-l-violet-500/50 lg:col-span-2">
                        <CardHeader>
                            <div className="flex items-center gap-3">
                                <div className="rounded-lg bg-violet-500/10 p-2 dark:bg-violet-500/15">
                                    <Wallet className="size-4 text-violet-600 dark:text-violet-400" />
                                </div>
                                <CardTitle>Assigned Wallets</CardTitle>
                            </div>
                        </CardHeader>
                        <CardContent>
                            {!strategy.wallet_strategies?.length ? (
                                <div className="py-6 text-center">
                                    <p className="text-sm text-muted-foreground">
                                        No wallets assigned yet.
                                    </p>
                                </div>
                            ) : (
                                <div className="divide-y">
                                    {strategy.wallet_strategies.map((ws) => (
                                        <div key={ws.id} className="flex items-center justify-between py-3 first:pt-0 last:pb-0">
                                            <div className="flex items-center gap-2 truncate">
                                                <span className="truncate font-mono text-sm">
                                                    {ws.wallet.label || `${(ws.wallet.safe_address ?? ws.wallet.signer_address).slice(0, 10)}...`}
                                                </span>
                                                {ws.is_paper && (
                                                    <Badge variant="outline" className="shrink-0 border-amber-500/50 text-amber-600 dark:text-amber-400">
                                                        Paper
                                                    </Badge>
                                                )}
                                            </div>
                                            <span
                                                className={`shrink-0 text-xs font-semibold ${
                                                    ws.is_running
                                                        ? 'text-emerald-600 dark:text-emerald-400'
                                                        : 'text-muted-foreground'
                                                }`}
                                            >
                                                {ws.is_running ? 'Running' : 'Stopped'}
                                            </span>
                                        </div>
                                    ))}
                                </div>
                            )}
                        </CardContent>
                    </Card>
                </div>

                <Deferred data={['liveStats', 'recentTrades']} fallback={<LiveDataSkeleton />}>
                    <Tabs defaultValue="live" className="mt-6">
                        <TabsList>
                            <TabsTrigger value="live">Live</TabsTrigger>
                            <TabsTrigger value="paper">Paper</TabsTrigger>
                        </TabsList>
                        <TabsContent value="live">
                            <div className="grid gap-4 sm:grid-cols-3">
                                <MetricCard
                                    label="Total Trades"
                                    value={currentStats?.total_trades ?? 0}
                                    icon={Activity}
                                />
                                <MetricCard
                                    label="Win Rate"
                                    value={formatWinRate(currentStats?.win_rate ?? null)}
                                    icon={TrendingUp}
                                />
                                <MetricCard
                                    label="PnL"
                                    value={formatPnl(currentStats?.total_pnl_usdc ?? null)}
                                    icon={ArrowLeftRight}
                                    trend={currentStats?.total_pnl_usdc ? pnlTrend : undefined}
                                />
                            </div>
                        </TabsContent>
                        <TabsContent value="paper">
                            <div className="grid gap-4 sm:grid-cols-3">
                                <MetricCard
                                    label="Total Trades"
                                    value={paperStats?.total_trades ?? 0}
                                    icon={Activity}
                                />
                                <MetricCard
                                    label="Win Rate"
                                    value={formatWinRate(paperStats?.win_rate ?? null)}
                                    icon={TrendingUp}
                                />
                                <MetricCard
                                    label="PnL"
                                    value={formatPnl(paperStats?.total_pnl_usdc ?? null)}
                                    icon={ArrowLeftRight}
                                    trend={paperStats?.total_pnl_usdc ? paperPnlTrend : undefined}
                                />
                            </div>
                        </TabsContent>
                    </Tabs>

                    <Card className="mt-6 border-l-4 border-l-emerald-500/50">
                        <CardHeader>
                            <div className="flex items-center gap-3">
                                <div className="rounded-lg bg-emerald-500/10 p-2 dark:bg-emerald-500/15">
                                    <ArrowLeftRight className="size-4 text-emerald-600 dark:text-emerald-400" />
                                </div>
                                <CardTitle>Recent Trades</CardTitle>
                            </div>
                        </CardHeader>
                        <CardContent>
                            {!recentTrades?.length ? (
                                <p className="text-sm text-muted-foreground">No trades yet.</p>
                            ) : (
                                <Table>
                                    <TableHeader>
                                        <TableRow>
                                            <TableHead>Date</TableHead>
                                            <TableHead>Market</TableHead>
                                            <TableHead>Side</TableHead>
                                            <TableHead>Outcome</TableHead>
                                            <TableHead>Price</TableHead>
                                            <TableHead>Size</TableHead>
                                            <TableHead>Type</TableHead>
                                            <TableHead>Status</TableHead>
                                        </TableRow>
                                    </TableHeader>
                                    <TableBody>
                                        {recentTrades.map((trade) => (
                                            <TableRow key={trade.id}>
                                                <TableCell className="text-muted-foreground">
                                                    {trade.executed_at
                                                        ? new Date(trade.executed_at).toLocaleDateString()
                                                        : '-'}
                                                </TableCell>
                                                <TableCell className="max-w-[200px] truncate font-mono text-xs">
                                                    {trade.market_id ?? '-'}
                                                </TableCell>
                                                <TableCell>
                                                    <span className={
                                                        trade.side === 'buy'
                                                            ? 'text-emerald-600 dark:text-emerald-400'
                                                            : 'text-red-500 dark:text-red-400'
                                                    }>
                                                        {trade.side?.toUpperCase() ?? '-'}
                                                    </span>
                                                </TableCell>
                                                <TableCell>{trade.outcome ?? '-'}</TableCell>
                                                <TableCell className="tabular-nums">
                                                    {trade.price ? `$${parseFloat(trade.price).toFixed(4)}` : '-'}
                                                </TableCell>
                                                <TableCell className="tabular-nums">
                                                    {formatPnl(trade.size_usdc)}
                                                </TableCell>
                                                <TableCell>
                                                    <span className={`inline-flex rounded-full px-2 py-0.5 text-xs font-medium ${
                                                        trade.is_paper
                                                            ? 'bg-amber-500/10 text-amber-700 dark:text-amber-400'
                                                            : 'bg-blue-500/10 text-blue-700 dark:text-blue-400'
                                                    }`}>
                                                        {trade.is_paper ? 'Paper' : 'Live'}
                                                    </span>
                                                </TableCell>
                                                <TableCell>
                                                    <span className={`inline-flex rounded-full px-2 py-0.5 text-xs font-medium ${
                                                        trade.status === 'filled'
                                                            ? 'bg-emerald-500/10 text-emerald-700 dark:text-emerald-400'
                                                            : trade.status === 'pending'
                                                              ? 'bg-amber-500/10 text-amber-700 dark:text-amber-400'
                                                              : 'bg-muted text-muted-foreground'
                                                    }`}>
                                                        {trade.status}
                                                    </span>
                                                </TableCell>
                                            </TableRow>
                                        ))}
                                    </TableBody>
                                </Table>
                            )}
                        </CardContent>
                    </Card>
                </Deferred>

                <Card className="mt-6 border-l-4 border-l-amber-500/50">
                    <CardHeader>
                        <div className="flex items-center gap-3">
                            <div className="rounded-lg bg-amber-500/10 p-2 dark:bg-amber-500/15">
                                <LineChart className="size-4 text-amber-600 dark:text-amber-400" />
                            </div>
                            <CardTitle>Recent Backtests</CardTitle>
                        </div>
                    </CardHeader>
                    <CardContent>
                        {!strategy.backtest_results?.length ? (
                            <p className="text-sm text-muted-foreground">No backtests yet.</p>
                        ) : (
                            <BacktestResultsTable results={strategy.backtest_results} />
                        )}
                    </CardContent>
                </Card>

                <Card className="mt-6 border-l-4 border-l-cyan-500/50">
                    <CardHeader>
                        <div className="flex items-center gap-3">
                            <div className="rounded-lg bg-cyan-500/10 p-2 dark:bg-cyan-500/15">
                                <FlaskConical className="size-4 text-cyan-600 dark:text-cyan-400" />
                            </div>
                            <CardTitle>Run Backtest</CardTitle>
                        </div>
                    </CardHeader>
                    <CardContent>
                        <form onSubmit={handleBacktestSubmit} className="space-y-6">
                            <div className="grid gap-6 sm:grid-cols-2 xl:grid-cols-3">
                                <div className="space-y-2">
                                    <Label htmlFor="date_from">Date From</Label>
                                    <Input
                                        id="date_from"
                                        type="date"
                                        value={backtestForm.data.date_from}
                                        onChange={(e) => backtestForm.setData('date_from', e.target.value)}
                                    />
                                    <InputError message={backtestForm.errors.date_from} />
                                </div>
                                <div className="space-y-2">
                                    <Label htmlFor="date_to">Date To</Label>
                                    <Input
                                        id="date_to"
                                        type="date"
                                        value={backtestForm.data.date_to}
                                        onChange={(e) => backtestForm.setData('date_to', e.target.value)}
                                    />
                                    <InputError message={backtestForm.errors.date_to} />
                                </div>
                            </div>
                            <Button type="submit" size="lg" disabled={backtestForm.processing}>
                                {backtestForm.processing ? 'Running...' : 'Run Backtest'}
                            </Button>
                        </form>
                    </CardContent>
                </Card>
            </div>
        </AppLayout>
    );
}
