import { Head, router, useForm } from '@inertiajs/react';
import { FlaskConical, LineChart, Settings2, Wallet } from 'lucide-react';
import AppLayout from '@/layouts/app-layout';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import InputError from '@/components/input-error';
import ConfirmDialog from '@/components/confirm-dialog';
import StatusBadge from '@/components/status-badge';
import BacktestResultsTable from '@/components/backtest-results-table';
import StrategyRulesDisplay, { isFormModeGraph } from '@/components/strategy/strategy-rules-display';
import type { BreadcrumbItem } from '@/types';
import type { Strategy } from '@/types/models';
import { index, show, activate, deactivate, destroy } from '@/actions/App/Http/Controllers/StrategyController';
import { run as runBacktest } from '@/actions/App/Http/Controllers/BacktestController';

export default function StrategiesShow({ strategy }: { strategy: Strategy }) {
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
                                            <span className="truncate font-mono text-sm">
                                                {ws.wallet.label || `${ws.wallet.address.slice(0, 10)}...`}
                                            </span>
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
