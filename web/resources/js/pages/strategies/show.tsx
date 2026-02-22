import { Head, router, useForm } from '@inertiajs/react';
import AppLayout from '@/layouts/app-layout';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import InputError from '@/components/input-error';
import ConfirmDialog from '@/components/confirm-dialog';
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

                <div className="grid gap-6 md:grid-cols-2">
                    <Card>
                        <CardHeader>
                            <CardTitle>Configuration</CardTitle>
                        </CardHeader>
                        <CardContent>
                            <dl className="space-y-2 text-sm">
                                <div className="flex justify-between">
                                    <dt className="text-muted-foreground">Mode</dt>
                                    <dd>{strategy.mode}</dd>
                                </div>
                                <div className="flex justify-between">
                                    <dt className="text-muted-foreground">Status</dt>
                                    <dd>{strategy.is_active ? 'Active' : 'Inactive'}</dd>
                                </div>
                            </dl>

                            {strategy.graph && isFormModeGraph(strategy.graph as Record<string, unknown>) && (
                                <StrategyRulesDisplay graph={strategy.graph} />
                            )}
                        </CardContent>
                    </Card>

                    <Card>
                        <CardHeader>
                            <CardTitle>Assigned Wallets</CardTitle>
                        </CardHeader>
                        <CardContent>
                            {!strategy.wallet_strategies?.length ? (
                                <p className="text-sm text-muted-foreground">
                                    No wallets assigned.
                                </p>
                            ) : (
                                <ul className="space-y-2 text-sm">
                                    {strategy.wallet_strategies.map((ws) => (
                                        <li key={ws.id} className="flex items-center justify-between">
                                            <span className="font-mono text-xs">
                                                {ws.wallet.label || `${ws.wallet.address.slice(0, 10)}...`}
                                            </span>
                                            <span className={ws.is_running ? 'text-green-600' : 'text-gray-400'}>
                                                {ws.is_running ? 'Running' : 'Stopped'}
                                            </span>
                                        </li>
                                    ))}
                                </ul>
                            )}
                        </CardContent>
                    </Card>
                </div>

                <Card className="mt-6">
                    <CardHeader>
                        <CardTitle>Recent Backtests</CardTitle>
                    </CardHeader>
                    <CardContent>
                        {!strategy.backtest_results?.length ? (
                            <p className="text-sm text-muted-foreground">No backtests yet.</p>
                        ) : (
                            <BacktestResultsTable results={strategy.backtest_results} />
                        )}
                    </CardContent>
                </Card>

                <Card className="mt-6">
                    <CardHeader>
                        <CardTitle>Run Backtest</CardTitle>
                    </CardHeader>
                    <CardContent>
                        <form onSubmit={handleBacktestSubmit} className="space-y-4">
                            <div className="grid gap-4 sm:grid-cols-2">
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
                            <Button type="submit" disabled={backtestForm.processing}>
                                {backtestForm.processing ? 'Running...' : 'Run Backtest'}
                            </Button>
                        </form>
                    </CardContent>
                </Card>
            </div>
        </AppLayout>
    );
}
