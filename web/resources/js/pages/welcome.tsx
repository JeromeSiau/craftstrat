import { Head, Link, usePage } from '@inertiajs/react';
import {
    ArrowRight,
    BarChart3,
    Blocks,
    Check,
    Copy,
    LineChart,
    Wallet,
    Zap,
} from 'lucide-react';
import type { RefCallback } from 'react';
import { useEffect, useState } from 'react';
import AppLogoIcon from '@/components/app-logo-icon';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { cn } from '@/lib/utils';
import { dashboard, login, register } from '@/routes';

/* ------------------------------------------------------------------ */
/*  Data                                                               */
/* ------------------------------------------------------------------ */

const features = [
    {
        icon: Blocks,
        title: 'No-Code Strategy Builder',
        description:
            'Design complex trading rules with the visual form builder, or go deeper with the node-based graph editor.',
    },
    {
        icon: Zap,
        title: 'Real-Time Execution',
        description:
            'Strategies execute in milliseconds across multiple Polygon wallets simultaneously. 24/7, fully automated.',
    },
    {
        icon: BarChart3,
        title: 'Historical Backtesting',
        description:
            'Test your strategies against real Polymarket order book data before risking real capital.',
    },
    {
        icon: Copy,
        title: 'Copy Trading',
        description:
            'Follow any public Polymarket wallet. Mirror trades automatically with full slippage tracking.',
    },
    {
        icon: Wallet,
        title: 'Multi-Wallet Management',
        description:
            'Generate and manage multiple Polygon wallets. Assign different strategies to each one independently.',
    },
    {
        icon: LineChart,
        title: 'Advanced Analytics',
        description:
            'Track win rates, PnL, drawdowns, and market calibration with rich visual dashboards.',
    },
];

const stats = [
    { value: '<50ms', label: 'Execution Speed' },
    { value: '24/7', label: 'Automated Trading' },
    { value: '1M+', label: 'Data Points Processed' },
    { value: '99.9%', label: 'Uptime' },
];

const steps = [
    {
        number: '01',
        title: 'Build',
        description:
            'Create your trading strategy using our visual no-code builder. Define conditions, indicators, and execution rules.',
    },
    {
        number: '02',
        title: 'Backtest',
        description:
            'Test against real historical Polymarket data. Analyze win rates, PnL curves, and risk metrics before going live.',
    },
    {
        number: '03',
        title: 'Deploy',
        description:
            'Activate your strategy across your wallets and let it run 24/7. Monitor performance in real-time.',
    },
];

const plans = [
    {
        name: 'Free',
        price: '$0',
        period: 'forever',
        features: [
            '1 wallet',
            '2 strategies',
            '30-day backtest history',
            '1 copy leader',
        ],
    },
    {
        name: 'Starter',
        price: '$29',
        period: '/mo',
        features: [
            '5 wallets',
            '10 strategies',
            'Full backtest history',
            '5 copy leaders',
            'Revenue sharing',
        ],
    },
    {
        name: 'Pro',
        price: '$79',
        period: '/mo',
        popular: true,
        features: [
            '25 wallets',
            'Unlimited strategies',
            'Full backtest history',
            'Unlimited copy trading',
            'Revenue sharing',
        ],
    },
    {
        name: 'Enterprise',
        price: '$249',
        period: '/mo',
        features: [
            'Unlimited wallets',
            'Unlimited strategies',
            'Full history + API access',
            'Custom leader fees',
            'Revenue sharing',
        ],
    },
];

const tickerItems = [
    { event: 'BTC > $100K by March', odds: '0.72', direction: 'up' as const },
    { event: 'Fed Rate Cut Q1', odds: '0.34', direction: 'down' as const },
    { event: 'ETH Flip BTC', odds: '0.08', direction: 'down' as const },
    { event: 'GPT-5 Release 2026', odds: '0.61', direction: 'up' as const },
    { event: 'Tesla $500', odds: '0.45', direction: 'up' as const },
    { event: 'US Recession 2026', odds: '0.28', direction: 'down' as const },
    { event: 'Mars Mission 2030', odds: '0.52', direction: 'up' as const },
    { event: 'Gold > $3K', odds: '0.67', direction: 'up' as const },
];

/* ------------------------------------------------------------------ */
/*  Hooks                                                              */
/* ------------------------------------------------------------------ */

function useInView(threshold = 0.15): [RefCallback<HTMLElement>, boolean] {
    const [el, setEl] = useState<HTMLElement | null>(null);
    const [inView, setInView] = useState(false);

    useEffect(() => {
        if (!el) return;
        const observer = new IntersectionObserver(
            ([entry]) => {
                if (entry.isIntersecting) setInView(true);
            },
            { threshold },
        );
        observer.observe(el);
        return () => observer.disconnect();
    }, [el, threshold]);

    return [setEl, inView];
}

/* ------------------------------------------------------------------ */
/*  Component                                                          */
/* ------------------------------------------------------------------ */

export default function Welcome({
    canRegister = true,
}: {
    canRegister?: boolean;
}) {
    const { auth } = usePage<{ auth: { user: unknown } }>().props;

    const [statsRef, statsInView] = useInView(0.2);
    const [featuresRef, featuresInView] = useInView();
    const [stepsRef, stepsInView] = useInView();
    const [pricingRef, pricingInView] = useInView();

    return (
        <>
            <Head title="Oddex — Prediction Market Trading Engine">
                <link rel="preconnect" href="https://fonts.bunny.net" />
                <link
                    href="https://fonts.bunny.net/css?family=instrument-sans:400,500,600|syne:700,800|jetbrains-mono:400"
                    rel="stylesheet"
                />
            </Head>

            <div className="min-h-screen bg-background text-foreground">
                {/* ============ NAV ============ */}
                <nav className="sticky top-0 z-50 border-b border-border/50 bg-background/80 backdrop-blur-xl">
                    <div className="mx-auto flex h-16 max-w-7xl items-center justify-between px-4 sm:px-6 lg:px-8">
                        <Link href="/" className="flex items-center gap-2.5">
                            <div className="flex size-8 items-center justify-center rounded-lg bg-primary">
                                <AppLogoIcon className="size-4.5 fill-current text-primary-foreground" />
                            </div>
                            <span
                                className="text-lg font-bold tracking-tight"
                                style={{ fontFamily: 'Syne, sans-serif' }}
                            >
                                Oddex
                            </span>
                        </Link>

                        <div className="hidden items-center gap-8 text-sm text-muted-foreground md:flex">
                            <a
                                href="#features"
                                className="transition hover:text-foreground"
                            >
                                Features
                            </a>
                            <a
                                href="#how-it-works"
                                className="transition hover:text-foreground"
                            >
                                How It Works
                            </a>
                            <a
                                href="#pricing"
                                className="transition hover:text-foreground"
                            >
                                Pricing
                            </a>
                        </div>

                        <div className="flex items-center gap-3">
                            {auth.user ? (
                                <Button asChild>
                                    <Link href={dashboard()}>Dashboard</Link>
                                </Button>
                            ) : (
                                <>
                                    <Button variant="ghost" size="sm" asChild>
                                        <Link href={login()}>Log in</Link>
                                    </Button>
                                    {canRegister && (
                                        <Button size="sm" asChild>
                                            <Link href={register()}>
                                                Get Started
                                            </Link>
                                        </Button>
                                    )}
                                </>
                            )}
                        </div>
                    </div>
                </nav>

                {/* ============ HERO ============ */}
                <section className="relative overflow-hidden">
                    {/* Dot grid */}
                    <div className="dot-grid absolute inset-0" />

                    {/* Amber glow */}
                    <div className="pointer-events-none absolute top-1/3 left-1/2 h-[500px] w-[700px] -translate-x-1/2 -translate-y-1/2 rounded-full bg-primary/10 blur-[120px]" />

                    <div className="relative mx-auto max-w-7xl px-4 pt-24 pb-20 sm:px-6 sm:pt-32 lg:px-8 lg:pt-40">
                        <div className="mx-auto max-w-4xl text-center">
                            {/* Tag */}
                            <div
                                className="mb-6 inline-flex items-center gap-2 rounded-full border border-primary/20 bg-primary/5 px-4 py-1.5 text-xs font-medium tracking-widest text-primary uppercase"
                                style={{
                                    fontFamily: 'JetBrains Mono, monospace',
                                    animation:
                                        'fade-up 0.7s ease-out 0.1s backwards',
                                }}
                            >
                                <span className="size-1.5 animate-pulse rounded-full bg-primary" />
                                Prediction Market Automation
                            </div>

                            {/* Headline */}
                            <h1
                                className="text-5xl font-extrabold tracking-tight sm:text-6xl lg:text-7xl xl:text-8xl"
                                style={{
                                    fontFamily: 'Syne, sans-serif',
                                    animation:
                                        'fade-up 0.8s ease-out 0.2s backwards',
                                }}
                            >
                                Your Edge,{' '}
                                <span className="text-primary">Automated</span>
                            </h1>

                            {/* Description */}
                            <p
                                className="mx-auto mt-6 max-w-2xl text-lg text-muted-foreground sm:text-xl"
                                style={{
                                    animation:
                                        'fade-up 0.8s ease-out 0.35s backwards',
                                }}
                            >
                                Build trading strategies without code, backtest
                                against real Polymarket data, and deploy across
                                multiple wallets — all from one platform.
                            </p>

                            {/* CTAs */}
                            <div
                                className="mt-10 flex flex-col items-center justify-center gap-4 sm:flex-row"
                                style={{
                                    animation:
                                        'fade-up 0.8s ease-out 0.5s backwards',
                                }}
                            >
                                {canRegister && (
                                    <Button
                                        size="lg"
                                        className="h-12 px-8 text-base"
                                        asChild
                                    >
                                        <Link href={register()}>
                                            Start Trading Free
                                            <ArrowRight className="ml-1 size-4" />
                                        </Link>
                                    </Button>
                                )}
                                <Button
                                    variant="outline"
                                    size="lg"
                                    className="h-12 px-8 text-base"
                                    asChild
                                >
                                    <a href="#how-it-works">See How It Works</a>
                                </Button>
                            </div>
                        </div>
                    </div>

                    {/* Ticker strip */}
                    <div className="relative overflow-hidden border-y border-border/50 bg-muted/30 py-3">
                        <div
                            className="flex"
                            style={{
                                animation: 'ticker-scroll 50s linear infinite',
                                fontFamily: 'JetBrains Mono, monospace',
                            }}
                        >
                            {[...tickerItems, ...tickerItems].map((item, i) => (
                                <div
                                    key={i}
                                    className="flex shrink-0 items-center gap-5 px-6"
                                >
                                    <span className="text-xs whitespace-nowrap text-muted-foreground">
                                        {item.event}
                                    </span>
                                    <span
                                        className={cn(
                                            'text-xs font-medium',
                                            item.direction === 'up'
                                                ? 'text-chart-3'
                                                : 'text-chart-5',
                                        )}
                                    >
                                        {item.direction === 'up' ? '↑' : '↓'}{' '}
                                        {item.odds}
                                    </span>
                                    <span className="text-border">│</span>
                                </div>
                            ))}
                        </div>
                    </div>
                </section>

                {/* ============ STATS ============ */}
                <section ref={statsRef} className="border-b border-border/50">
                    <div className="mx-auto grid max-w-7xl grid-cols-2 sm:grid-cols-4">
                        {stats.map((stat, i) => (
                            <div
                                key={stat.label}
                                className={cn(
                                    'flex flex-col items-center gap-1 px-6 py-12 text-center transition-all duration-700',
                                    statsInView
                                        ? 'translate-y-0 opacity-100'
                                        : 'translate-y-4 opacity-0',
                                )}
                                style={{ transitionDelay: `${i * 100}ms` }}
                            >
                                <span
                                    className="text-3xl font-bold text-primary sm:text-4xl"
                                    style={{
                                        fontFamily: 'JetBrains Mono, monospace',
                                    }}
                                >
                                    {stat.value}
                                </span>
                                <span className="text-sm text-muted-foreground">
                                    {stat.label}
                                </span>
                            </div>
                        ))}
                    </div>
                </section>

                {/* ============ FEATURES ============ */}
                <section
                    id="features"
                    ref={featuresRef}
                    className="py-24 sm:py-32"
                >
                    <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
                        <div className="mx-auto max-w-2xl text-center">
                            <h2
                                className="text-3xl font-bold tracking-tight sm:text-4xl"
                                style={{ fontFamily: 'Syne, sans-serif' }}
                            >
                                Everything You Need to{' '}
                                <span className="text-primary">Win</span>
                            </h2>
                            <p className="mt-4 text-lg text-muted-foreground">
                                Professional-grade tools designed for prediction
                                market traders.
                            </p>
                        </div>

                        <div className="mt-16 grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
                            {features.map((feature, i) => (
                                <div
                                    key={feature.title}
                                    className={cn(
                                        'group relative rounded-xl border border-border/50 bg-card/50 p-6 transition-all duration-500',
                                        'hover:border-primary/30 hover:shadow-lg hover:shadow-primary/5',
                                        featuresInView
                                            ? 'translate-y-0 opacity-100'
                                            : 'translate-y-8 opacity-0',
                                    )}
                                    style={{
                                        transitionDelay: `${i * 80}ms`,
                                    }}
                                >
                                    <div className="mb-4 inline-flex rounded-lg bg-primary/10 p-2.5 text-primary transition-colors group-hover:bg-primary/15">
                                        <feature.icon className="size-5" />
                                    </div>
                                    <h3 className="mb-2 font-semibold">
                                        {feature.title}
                                    </h3>
                                    <p className="text-sm leading-relaxed text-muted-foreground">
                                        {feature.description}
                                    </p>
                                </div>
                            ))}
                        </div>
                    </div>
                </section>

                {/* ============ HOW IT WORKS ============ */}
                <section
                    id="how-it-works"
                    ref={stepsRef}
                    className="border-y border-border/50 bg-muted/20 py-24 sm:py-32"
                >
                    <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
                        <div className="mx-auto max-w-2xl text-center">
                            <h2
                                className="text-3xl font-bold tracking-tight sm:text-4xl"
                                style={{ fontFamily: 'Syne, sans-serif' }}
                            >
                                Three Steps to{' '}
                                <span className="text-primary">
                                    Automated Trading
                                </span>
                            </h2>
                            <p className="mt-4 text-lg text-muted-foreground">
                                Go from idea to live execution in minutes, not
                                days.
                            </p>
                        </div>

                        <div className="relative mt-16 grid gap-12 lg:grid-cols-3 lg:gap-8">
                            {/* Connector line (desktop) */}
                            <div className="absolute top-8 right-[calc(16.67%+16px)] left-[calc(16.67%+16px)] hidden h-px bg-gradient-to-r from-primary/30 via-primary/15 to-primary/30 lg:block" />

                            {steps.map((step, i) => (
                                <div
                                    key={step.number}
                                    className={cn(
                                        'relative text-center transition-all duration-700',
                                        stepsInView
                                            ? 'translate-y-0 opacity-100'
                                            : 'translate-y-8 opacity-0',
                                    )}
                                    style={{
                                        transitionDelay: `${i * 150}ms`,
                                    }}
                                >
                                    <div
                                        className="relative z-10 mx-auto mb-6 flex size-16 items-center justify-center rounded-2xl border border-primary/20 bg-background text-2xl font-bold text-primary"
                                        style={{
                                            fontFamily:
                                                'JetBrains Mono, monospace',
                                        }}
                                    >
                                        {step.number}
                                    </div>
                                    <h3 className="mb-3 text-xl font-semibold">
                                        {step.title}
                                    </h3>
                                    <p className="mx-auto max-w-xs text-sm leading-relaxed text-muted-foreground">
                                        {step.description}
                                    </p>
                                </div>
                            ))}
                        </div>
                    </div>
                </section>

                {/* ============ PRICING ============ */}
                <section
                    id="pricing"
                    ref={pricingRef}
                    className="py-24 sm:py-32"
                >
                    <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
                        <div className="mx-auto max-w-2xl text-center">
                            <h2
                                className="text-3xl font-bold tracking-tight sm:text-4xl"
                                style={{ fontFamily: 'Syne, sans-serif' }}
                            >
                                Simple, Transparent{' '}
                                <span className="text-primary">Pricing</span>
                            </h2>
                            <p className="mt-4 text-lg text-muted-foreground">
                                Start for free. Scale as you grow.
                            </p>
                        </div>

                        <div className="mt-16 grid gap-4 sm:grid-cols-2 xl:grid-cols-4">
                            {plans.map((plan, i) => (
                                <div
                                    key={plan.name}
                                    className={cn(
                                        'relative flex flex-col rounded-xl border p-6 transition-all duration-500',
                                        plan.popular
                                            ? 'border-primary/50 bg-primary/5 shadow-lg shadow-primary/10'
                                            : 'border-border/50 bg-card/50',
                                        pricingInView
                                            ? 'translate-y-0 opacity-100'
                                            : 'translate-y-8 opacity-0',
                                    )}
                                    style={{
                                        transitionDelay: `${i * 100}ms`,
                                    }}
                                >
                                    {plan.popular && (
                                        <Badge className="absolute -top-3 left-1/2 -translate-x-1/2">
                                            Most Popular
                                        </Badge>
                                    )}
                                    <h3 className="text-lg font-semibold">
                                        {plan.name}
                                    </h3>
                                    <div className="mt-4 flex items-baseline">
                                        <span
                                            className="text-4xl font-bold tracking-tight"
                                            style={{
                                                fontFamily: 'Syne, sans-serif',
                                            }}
                                        >
                                            {plan.price}
                                        </span>
                                        <span className="ml-1 text-muted-foreground">
                                            {plan.period}
                                        </span>
                                    </div>

                                    <ul className="mt-6 flex-1 space-y-3">
                                        {plan.features.map((feature) => (
                                            <li
                                                key={feature}
                                                className="flex items-start gap-2.5 text-sm"
                                            >
                                                <Check className="mt-0.5 size-4 shrink-0 text-primary" />
                                                <span>{feature}</span>
                                            </li>
                                        ))}
                                    </ul>

                                    <Button
                                        variant={
                                            plan.popular ? 'default' : 'outline'
                                        }
                                        className="mt-8 w-full"
                                        size="lg"
                                        asChild
                                    >
                                        <Link
                                            href={
                                                canRegister
                                                    ? register()
                                                    : login()
                                            }
                                        >
                                            {plan.price === '$0'
                                                ? 'Start Free'
                                                : 'Get Started'}
                                        </Link>
                                    </Button>
                                </div>
                            ))}
                        </div>
                    </div>
                </section>

                {/* ============ FINAL CTA ============ */}
                <section className="relative overflow-hidden border-t border-border/50 py-24 sm:py-32">
                    <div className="pointer-events-none absolute inset-0 bg-gradient-to-b from-transparent via-primary/5 to-transparent" />
                    <div className="pointer-events-none absolute top-1/2 left-1/2 h-[300px] w-[500px] -translate-x-1/2 -translate-y-1/2 rounded-full bg-primary/8 blur-[100px]" />

                    <div className="relative mx-auto max-w-3xl px-4 text-center sm:px-6 lg:px-8">
                        <h2
                            className="text-3xl font-bold tracking-tight sm:text-4xl lg:text-5xl"
                            style={{ fontFamily: 'Syne, sans-serif' }}
                        >
                            Ready to Trade{' '}
                            <span className="text-primary">Smarter</span>?
                        </h2>
                        <p className="mx-auto mt-6 max-w-xl text-lg text-muted-foreground">
                            Join traders who automate their prediction market
                            strategies with Oddex. Start for free — no credit
                            card required.
                        </p>
                        {canRegister && (
                            <Button
                                size="lg"
                                className="mt-10 h-12 px-8 text-base"
                                asChild
                            >
                                <Link href={register()}>
                                    Create Free Account
                                    <ArrowRight className="ml-1 size-4" />
                                </Link>
                            </Button>
                        )}
                    </div>
                </section>

                {/* ============ FOOTER ============ */}
                <footer className="border-t border-border/50">
                    <div className="mx-auto flex max-w-7xl flex-col items-center justify-between gap-6 px-4 py-12 sm:flex-row sm:px-6 lg:px-8">
                        <div className="flex items-center gap-2.5">
                            <div className="flex size-7 items-center justify-center rounded-md bg-primary">
                                <AppLogoIcon className="size-3.5 fill-current text-primary-foreground" />
                            </div>
                            <span
                                className="font-bold"
                                style={{ fontFamily: 'Syne, sans-serif' }}
                            >
                                Oddex
                            </span>
                        </div>

                        <div className="flex items-center gap-6 text-sm text-muted-foreground">
                            <a
                                href="#features"
                                className="transition hover:text-foreground"
                            >
                                Features
                            </a>
                            <a
                                href="#pricing"
                                className="transition hover:text-foreground"
                            >
                                Pricing
                            </a>
                            {auth.user ? (
                                <Link
                                    href={dashboard()}
                                    className="transition hover:text-foreground"
                                >
                                    Dashboard
                                </Link>
                            ) : (
                                <Link
                                    href={login()}
                                    className="transition hover:text-foreground"
                                >
                                    Log in
                                </Link>
                            )}
                        </div>

                        <p className="text-xs text-muted-foreground">
                            &copy; {new Date().getFullYear()} Oddex. All rights
                            reserved.
                        </p>
                    </div>
                </footer>
            </div>
        </>
    );
}
