import { Head, Link } from '@inertiajs/react';
import AppLayout from '@/layouts/app-layout';
import { Button } from '@/components/ui/button';
import StatusBadge from '@/components/status-badge';
import type { BreadcrumbItem } from '@/types';
import type { Strategy, Paginated } from '@/types/models';
import { index, show, create } from '@/actions/App/Http/Controllers/StrategyController';

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
            <div className="p-6">
                <div className="mb-6 flex items-center justify-between">
                    <h1 className="text-2xl font-bold">Strategies</h1>
                    <Link href={create.url()}>
                        <Button>New Strategy</Button>
                    </Link>
                </div>
                <div className="space-y-3">
                    {strategies.data.length === 0 && (
                        <p className="text-muted-foreground">
                            No strategies yet. Create your first one.
                        </p>
                    )}
                    {strategies.data.map((strategy) => (
                        <Link
                            key={strategy.id}
                            href={show.url(strategy.id)}
                            className="block rounded-lg border p-4 transition hover:bg-accent"
                        >
                            <div className="flex items-center justify-between">
                                <div>
                                    <h3 className="font-semibold">
                                        {strategy.name}
                                    </h3>
                                    <p className="text-sm text-muted-foreground">
                                        {strategy.mode} mode ·{' '}
                                        {strategy.wallets_count ?? 0} wallet(s)
                                    </p>
                                </div>
                                <StatusBadge active={strategy.is_active} />
                            </div>
                        </Link>
                    ))}
                </div>
            </div>
        </AppLayout>
    );
}
