import { Head, router } from '@inertiajs/react';
import AppLayout from '@/layouts/app-layout';
import { Button } from '@/components/ui/button';
import type { BreadcrumbItem } from '@/types';
import { index, portal } from '@/actions/App/Http/Controllers/BillingController';

const breadcrumbs: BreadcrumbItem[] = [{ title: 'Billing', href: index.url() }];

const plans = [
    {
        key: 'free',
        name: 'Free',
        price: '$0/mo',
        wallets: '1',
        strategies: '2',
    },
    {
        key: 'starter',
        name: 'Starter',
        price: '$29/mo',
        wallets: '5',
        strategies: '10',
    },
    {
        key: 'pro',
        name: 'Pro',
        price: '$79/mo',
        wallets: '25',
        strategies: 'Unlimited',
    },
    {
        key: 'enterprise',
        name: 'Enterprise',
        price: '$249/mo',
        wallets: 'Unlimited',
        strategies: 'Unlimited',
    },
];

interface Props {
    plan: string;
    subscribed: boolean;
}

export default function BillingIndex({ plan, subscribed }: Props) {
    return (
        <AppLayout breadcrumbs={breadcrumbs}>
            <Head title="Billing" />
            <div className="p-6">
                <h1 className="mb-6 text-2xl font-bold">Billing</h1>

                <div className="mb-6 grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
                    {plans.map((p) => (
                        <div
                            key={p.key}
                            className={`rounded-xl border border-sidebar-border/70 p-4 dark:border-sidebar-border ${plan === p.key ? 'ring-2 ring-primary' : ''}`}
                        >
                            <h3 className="font-semibold">{p.name}</h3>
                            <p className="mt-1 text-2xl font-bold">{p.price}</p>
                            <ul className="mt-3 space-y-1 text-sm text-muted-foreground">
                                <li>{p.wallets} wallet(s)</li>
                                <li>{p.strategies} strategies</li>
                            </ul>
                            {plan === p.key && (
                                <p className="mt-3 text-sm font-medium text-primary">
                                    Current plan
                                </p>
                            )}
                        </div>
                    ))}
                </div>

                {subscribed && (
                    <Button
                        variant="outline"
                        onClick={() => router.post(portal.url())}
                    >
                        Manage Subscription
                    </Button>
                )}
            </div>
        </AppLayout>
    );
}
