import { Head, Link } from '@inertiajs/react';
import { ChevronRight, Plus, Target } from 'lucide-react';
import { index, show, create } from '@/actions/App/Http/Controllers/StrategyController';
import StatusBadge from '@/components/status-badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent } from '@/components/ui/card';
import AppLayout from '@/layouts/app-layout';
import { MARKET_LABEL_MAP } from '@/lib/constants';
import type { BreadcrumbItem } from '@/types';
import type { Strategy, Paginated } from '@/types/models';

const breadcrumbs: BreadcrumbItem[] = [
    { title: 'Strategies', href: index.url() },
];

export default function StrategiesIndex({
    strategies,
}: {
    strategies: Paginated<Strategy>;
}) {
    return (
        <AppLayout breadcrumbs={breadcrumbs}>
            <Head title="Strategies" />
            <div className="p-4 md:p-8">
                <div className="mb-8 flex items-center justify-between">
                    <div>
                        <h1 className="text-2xl font-bold tracking-tight">Strategies</h1>
                        <p className="mt-1 text-muted-foreground">
                            Manage your trading strategies and configurations.
                        </p>
                    </div>
                    <Link href={create.url()}>
                        <Button size="lg">
                            <Plus className="size-4" />
                            New Strategy
                        </Button>
                    </Link>
                </div>

                {strategies.data.length === 0 ? (
                    <Card>
                        <CardContent className="flex flex-col items-center justify-center py-16 text-center">
                            <div className="rounded-xl bg-muted p-4">
                                <Target className="size-8 text-muted-foreground" />
                            </div>
                            <p className="mt-4 font-medium">No strategies yet</p>
                            <p className="mt-1 text-sm text-muted-foreground">
                                Create your first strategy to start trading.
                            </p>
                            <Link href={create.url()} className="mt-6">
                                <Button>
                                    <Plus className="size-4" />
                                    Create Strategy
                                </Button>
                            </Link>
                        </CardContent>
                    </Card>
                ) : (
                    <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
                        {strategies.data.map((strategy) => (
                            <Link
                                key={strategy.id}
                                href={show.url(strategy.id)}
                                className="group"
                            >
                                <Card className="h-full transition hover:border-primary/30 hover:shadow-md">
                                    <CardContent className="flex h-full items-center justify-between gap-4 py-5">
                                        <div className="min-w-0">
                                            <div className="flex items-center gap-3">
                                                <h3 className="truncate font-semibold">
                                                    {strategy.name}
                                                </h3>
                                                <StatusBadge active={strategy.is_active} />
                                            </div>
                                            <p className="mt-1.5 text-sm text-muted-foreground">
                                                {strategy.mode} mode · {strategy.wallets_count ?? 0} wallet(s)
                                            </p>
                                            {(() => {
                                                const markets = [...new Set(strategy.wallet_strategies?.flatMap((ws) => ws.markets) ?? [])];
                                                return markets.length > 0 ? (
                                                    <div className="mt-2 flex flex-wrap gap-1">
                                                        {markets.map((m) => (
                                                            <span key={m} className="rounded-md bg-muted px-1.5 py-0.5 text-xs text-muted-foreground">
                                                                {MARKET_LABEL_MAP[m] ?? m}
                                                            </span>
                                                        ))}
                                                    </div>
                                                ) : null;
                                            })()}
                                        </div>
                                        <ChevronRight className="size-5 shrink-0 text-muted-foreground/50 transition group-hover:text-primary" />
                                    </CardContent>
                                </Card>
                            </Link>
                        ))}
                    </div>
                )}
            </div>
        </AppLayout>
    );
}
