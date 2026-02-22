import { Head, router } from '@inertiajs/react';
import AppLayout from '@/layouts/app-layout';
import { Button } from '@/components/ui/button';
import {
    Card,
    CardContent,
    CardHeader,
    CardTitle,
} from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Check } from 'lucide-react';
import type { BreadcrumbItem } from '@/types';
import {
    index,
    subscribe,
    portal,
} from '@/actions/App/Http/Controllers/BillingController';

const breadcrumbs: BreadcrumbItem[] = [
    { title: 'Billing', href: index.url() },
];

const plans = [
    {
        key: 'free',
        name: 'Free',
        price: '$0',
        period: 'forever',
        features: [
            '1 wallet',
            '2 strategies',
            '30-day backtest',
            '1 copy leader',
        ],
    },
    {
        key: 'starter',
        name: 'Starter',
        price: '$29',
        period: '/mo',
        priceId: 'price_starter',
        features: [
            '5 wallets',
            '10 strategies',
            'Full history backtest',
            '5 copy leaders',
            'Revenue sharing',
        ],
    },
    {
        key: 'pro',
        name: 'Pro',
        price: '$79',
        period: '/mo',
        priceId: 'price_pro',
        popular: true,
        features: [
            '25 wallets',
            'Unlimited strategies',
            'Full history backtest',
            'Unlimited copy + be leader',
            'Revenue sharing',
        ],
    },
    {
        key: 'enterprise',
        name: 'Enterprise',
        price: '$249',
        period: '/mo',
        priceId: 'price_enterprise',
        features: [
            'Unlimited wallets',
            'Unlimited strategies',
            'Full history + API',
            'Custom leader fees',
            'Revenue sharing',
        ],
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
                        <Card
                            key={p.key}
                            className={`relative ${plan === p.key ? 'ring-2 ring-primary' : ''}`}
                        >
                            {p.popular && (
                                <Badge className="absolute top-3 right-3">
                                    Popular
                                </Badge>
                            )}
                            <CardHeader>
                                <CardTitle>{p.name}</CardTitle>
                                <div className="mt-1">
                                    <span className="text-3xl font-bold">
                                        {p.price}
                                    </span>
                                    <span className="text-muted-foreground">
                                        {p.period}
                                    </span>
                                </div>
                            </CardHeader>
                            <CardContent>
                                <ul className="mb-4 space-y-2">
                                    {p.features.map((feature) => (
                                        <li
                                            key={feature}
                                            className="flex items-center gap-2 text-sm"
                                        >
                                            <Check className="h-4 w-4 shrink-0 text-primary" />
                                            {feature}
                                        </li>
                                    ))}
                                </ul>
                                {plan === p.key ? (
                                    <Button
                                        variant="outline"
                                        className="w-full"
                                        disabled
                                    >
                                        Current Plan
                                    </Button>
                                ) : p.priceId ? (
                                    <Button
                                        className="w-full"
                                        onClick={() =>
                                            router.post(subscribe.url(), {
                                                price_id: p.priceId,
                                            })
                                        }
                                    >
                                        {plan === 'free'
                                            ? 'Subscribe'
                                            : 'Upgrade'}
                                    </Button>
                                ) : null}
                            </CardContent>
                        </Card>
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
