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
            <div className="p-4 md:p-8">
                <div className="mb-8">
                    <h1 className="text-2xl font-bold tracking-tight">Billing</h1>
                    <p className="mt-1 text-muted-foreground">
                        Manage your subscription and billing details.
                    </p>
                </div>

                <div className="mb-8 grid gap-4 sm:grid-cols-2 xl:grid-cols-4">
                    {plans.map((p) => (
                        <Card
                            key={p.key}
                            className={`relative flex flex-col transition ${
                                plan === p.key
                                    ? 'ring-2 ring-primary shadow-lg'
                                    : p.popular
                                      ? 'border-primary/30'
                                      : ''
                            }`}
                        >
                            {p.popular && (
                                <Badge className="absolute top-4 right-4">
                                    Popular
                                </Badge>
                            )}
                            <CardHeader>
                                <CardTitle className="text-lg">{p.name}</CardTitle>
                                <div className="mt-2">
                                    <span className="text-4xl font-bold tracking-tight">
                                        {p.price}
                                    </span>
                                    <span className="ml-1 text-muted-foreground">
                                        {p.period}
                                    </span>
                                </div>
                            </CardHeader>
                            <CardContent className="flex flex-1 flex-col">
                                <ul className="mb-6 flex-1 space-y-3">
                                    {p.features.map((feature) => (
                                        <li
                                            key={feature}
                                            className="flex items-center gap-2.5 text-sm"
                                        >
                                            <Check className="size-4 shrink-0 text-primary" />
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
                                        size="lg"
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
                        size="lg"
                        onClick={() => router.post(portal.url())}
                    >
                        Manage Subscription
                    </Button>
                )}
            </div>
        </AppLayout>
    );
}
