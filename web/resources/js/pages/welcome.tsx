import { Head, Link, usePage } from '@inertiajs/react';
import { ArrowRight, Check, Copy, LineChart, Zap } from 'lucide-react';
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

const stats = [
    { value: '<50ms', label: 'Execution Speed' },
    { value: '24/7', label: 'Automated Trading' },
    { value: '500+', label: 'Markets Tracked' },
    { value: '99.9%', label: 'Uptime' },
];

const secondaryFeatures = [
    {
        icon: Copy,
        title: 'Copy Trading',
        description:
            'Follow any public Polymarket wallet. Mirror trades automatically with configurable position sizing.',
    },
    {
        icon: LineChart,
        title: 'Advanced Analytics',
        description:
            'Track win rates, PnL curves, drawdowns, and calibration metrics across all your strategies.',
    },
    {
        icon: Zap,
        title: 'Real-Time Execution',
        description:
            'Sub-50ms order placement across multiple wallets simultaneously. No manual intervention needed.',
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
/*  Shared style                                                       */
/* ------------------------------------------------------------------ */

const jakarta = { fontFamily: 'Plus Jakarta Sans, sans-serif' };

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
    const [builderRef, builderInView] = useInView();
    const [backtestRef, backtestInView] = useInView();
    const [walletsRef, walletsInView] = useInView();
    const [extrasRef, extrasInView] = useInView();
    const [pricingRef, pricingInView] = useInView();

    return (
        <>
            <Head title="CraftStrat — Polymarket Automated Trading">
                <link rel="preconnect" href="https://fonts.bunny.net" />
                <link
                    href="https://fonts.bunny.net/css?family=instrument-sans:400,500,600|plus-jakarta-sans:600,700,800"
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
                                style={jakarta}
                            >
                                CraftStrat
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
                    {/* Warm glow */}
                    <div className="pointer-events-none absolute top-1/3 left-1/2 h-[500px] w-[700px] -translate-x-1/2 -translate-y-1/2 rounded-full bg-primary/8 blur-[120px]" />

                    {/* Subtle horizontal lines */}
                    <div
                        className="pointer-events-none absolute inset-0 opacity-25"
                        style={{
                            backgroundImage:
                                'repeating-linear-gradient(0deg, transparent, transparent 79px, var(--border) 79px, var(--border) 80px)',
                        }}
                    />

                    <div className="relative mx-auto max-w-4xl px-4 pt-24 pb-20 text-center sm:px-6 sm:pt-32 lg:pt-40">
                        {/* Badge */}
                        <div
                            className="mb-6 inline-flex items-center gap-2 rounded-full border border-primary/20 bg-primary/5 px-4 py-1.5 text-xs font-medium tracking-widest text-primary uppercase"
                            style={{
                                ...jakarta,
                                animation:
                                    'fade-up 0.7s ease-out 0.1s backwards',
                            }}
                        >
                            <span className="size-1.5 animate-pulse rounded-full bg-primary" />
                            Polymarket Automation
                        </div>

                        {/* Headline */}
                        <h1
                            className="text-5xl font-extrabold tracking-tight sm:text-6xl lg:text-7xl"
                            style={{
                                ...jakarta,
                                animation:
                                    'fade-up 0.8s ease-out 0.2s backwards',
                            }}
                        >
                            Trade Polymarket
                            <br />
                            <span className="text-primary">on Autopilot</span>
                        </h1>

                        {/* Description */}
                        <p
                            className="mx-auto mt-6 max-w-2xl text-lg text-muted-foreground sm:text-xl"
                            style={{
                                animation:
                                    'fade-up 0.8s ease-out 0.35s backwards',
                            }}
                        >
                            Build strategies visually, backtest on real order
                            book data, and deploy across multiple wallets. No
                            code required.
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
                                <a href="#features">Explore Features</a>
                            </Button>
                        </div>
                    </div>
                </section>

                {/* ============ STATS ============ */}
                <section
                    ref={statsRef}
                    className="border-y border-border/50 bg-card"
                >
                    <div className="mx-auto grid max-w-7xl grid-cols-2 sm:grid-cols-4">
                        {stats.map((stat, i) => (
                            <div
                                key={stat.label}
                                className={cn(
                                    'flex flex-col items-center gap-1 border-border/50 px-6 py-12 text-center transition-all duration-700 sm:not-last:border-r',
                                    statsInView
                                        ? 'translate-y-0 opacity-100'
                                        : 'translate-y-4 opacity-0',
                                )}
                                style={{
                                    transitionDelay: `${i * 100}ms`,
                                }}
                            >
                                <span
                                    className="text-3xl font-bold text-primary sm:text-4xl"
                                    style={jakarta}
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

                {/* ============ FEATURE: Strategy Builder ============ */}
                <section
                    id="features"
                    ref={builderRef}
                    className="py-24 sm:py-32"
                >
                    <div
                        className={cn(
                            'mx-auto grid max-w-7xl items-center gap-12 px-4 sm:px-6 lg:grid-cols-2 lg:gap-20 lg:px-8',
                            'transition-all duration-700',
                            builderInView
                                ? 'translate-y-0 opacity-100'
                                : 'translate-y-8 opacity-0',
                        )}
                    >
                        {/* Text */}
                        <div>
                            <p
                                className="text-xs font-semibold tracking-widest text-primary uppercase"
                                style={jakarta}
                            >
                                Strategy Builder
                            </p>
                            <h2
                                className="mt-3 text-3xl font-bold tracking-tight sm:text-4xl"
                                style={jakarta}
                            >
                                Build strategies,
                                <br />
                                not spreadsheets
                            </h2>
                            <p className="mt-4 text-lg leading-relaxed text-muted-foreground">
                                Design trading rules with our visual form
                                builder — no code needed. For power users, the
                                node-based graph editor lets you wire complex
                                logic with full control.
                            </p>
                            <ul className="mt-6 space-y-3">
                                {[
                                    'Drag-and-drop condition blocks',
                                    'Advanced node/graph editor for complex logic',
                                    'Reusable strategy templates',
                                ].map((item) => (
                                    <li
                                        key={item}
                                        className="flex items-center gap-2.5 text-sm"
                                    >
                                        <Check className="size-4 shrink-0 text-primary" />
                                        <span>{item}</span>
                                    </li>
                                ))}
                            </ul>
                        </div>

                        {/* Mock UI */}
                        <div className="rounded-xl border border-border/50 bg-card p-6 shadow-sm">
                            <div className="mb-4 flex items-center gap-2">
                                <div className="size-2.5 rounded-full bg-red-400/60" />
                                <div className="size-2.5 rounded-full bg-amber-400/60" />
                                <div className="size-2.5 rounded-full bg-green-400/60" />
                                <span className="ml-2 text-xs text-muted-foreground">
                                    Strategy Editor
                                </span>
                            </div>
                            <div className="space-y-3">
                                <div className="rounded-lg border border-border/50 bg-muted/30 p-3">
                                    <div className="text-[10px] font-semibold tracking-widest text-muted-foreground uppercase">
                                        When
                                    </div>
                                    <div className="mt-1 text-sm font-medium">
                                        Market probability crosses above{' '}
                                        <span className="text-primary">
                                            0.65
                                        </span>
                                    </div>
                                </div>
                                <div className="flex justify-center text-muted-foreground/50">
                                    <svg
                                        className="size-4"
                                        fill="none"
                                        viewBox="0 0 24 24"
                                        stroke="currentColor"
                                        strokeWidth={2}
                                    >
                                        <path d="M12 5v14m0 0l-4-4m4 4l4-4" />
                                    </svg>
                                </div>
                                <div className="rounded-lg border border-primary/20 bg-primary/5 p-3">
                                    <div className="text-[10px] font-semibold tracking-widest text-primary uppercase">
                                        Then
                                    </div>
                                    <div className="mt-1 text-sm font-medium">
                                        Buy{' '}
                                        <span className="text-primary">
                                            $50
                                        </span>{' '}
                                        on YES
                                    </div>
                                </div>
                                <div className="flex justify-center text-muted-foreground/50">
                                    <svg
                                        className="size-4"
                                        fill="none"
                                        viewBox="0 0 24 24"
                                        stroke="currentColor"
                                        strokeWidth={2}
                                    >
                                        <path d="M12 5v14m0 0l-4-4m4 4l4-4" />
                                    </svg>
                                </div>
                                <div className="rounded-lg border border-border/50 bg-muted/30 p-3">
                                    <div className="text-[10px] font-semibold tracking-widest text-muted-foreground uppercase">
                                        Exit when
                                    </div>
                                    <div className="mt-1 text-sm font-medium">
                                        Profit reaches{' '}
                                        <span className="text-primary">
                                            +15%
                                        </span>{' '}
                                        or loss exceeds{' '}
                                        <span className="text-destructive">
                                            -8%
                                        </span>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                </section>

                {/* ============ FEATURE: Backtesting ============ */}
                <section
                    ref={backtestRef}
                    className="border-y border-border/50 bg-muted/20 py-24 sm:py-32"
                >
                    <div
                        className={cn(
                            'mx-auto grid max-w-7xl items-center gap-12 px-4 sm:px-6 lg:grid-cols-2 lg:gap-20 lg:px-8',
                            'transition-all duration-700',
                            backtestInView
                                ? 'translate-y-0 opacity-100'
                                : 'translate-y-8 opacity-0',
                        )}
                    >
                        {/* Mock UI (left on desktop) */}
                        <div className="order-2 rounded-xl border border-border/50 bg-card p-6 shadow-sm lg:order-1">
                            <div className="mb-4 flex items-center gap-2">
                                <div className="size-2.5 rounded-full bg-red-400/60" />
                                <div className="size-2.5 rounded-full bg-amber-400/60" />
                                <div className="size-2.5 rounded-full bg-green-400/60" />
                                <span className="ml-2 text-xs text-muted-foreground">
                                    Backtest Results
                                </span>
                            </div>
                            {/* Mock chart */}
                            <div className="relative h-32 w-full overflow-hidden rounded-lg bg-muted/30">
                                <svg
                                    className="absolute inset-0 size-full"
                                    viewBox="0 0 400 120"
                                    fill="none"
                                    preserveAspectRatio="none"
                                >
                                    <path
                                        d="M0 100 Q50 90 80 75 T160 60 T240 45 T320 55 T400 20"
                                        stroke="currentColor"
                                        className="text-primary/40"
                                        strokeWidth="2"
                                    />
                                    <path
                                        d="M0 100 Q50 90 80 75 T160 60 T240 45 T320 55 T400 20 V120 H0Z"
                                        className="fill-primary/5"
                                    />
                                </svg>
                            </div>
                            {/* Mock stats */}
                            <div className="mt-4 grid grid-cols-3 gap-4">
                                <div>
                                    <div
                                        className="text-lg font-bold text-primary"
                                        style={jakarta}
                                    >
                                        73%
                                    </div>
                                    <div className="text-[11px] text-muted-foreground">
                                        Win Rate
                                    </div>
                                </div>
                                <div>
                                    <div
                                        className="text-lg font-bold text-chart-3"
                                        style={jakarta}
                                    >
                                        +18.4%
                                    </div>
                                    <div className="text-[11px] text-muted-foreground">
                                        Return
                                    </div>
                                </div>
                                <div>
                                    <div
                                        className="text-lg font-bold"
                                        style={jakarta}
                                    >
                                        142
                                    </div>
                                    <div className="text-[11px] text-muted-foreground">
                                        Trades
                                    </div>
                                </div>
                            </div>
                        </div>

                        {/* Text (right on desktop) */}
                        <div className="order-1 lg:order-2">
                            <p
                                className="text-xs font-semibold tracking-widest text-primary uppercase"
                                style={jakarta}
                            >
                                Backtesting
                            </p>
                            <h2
                                className="mt-3 text-3xl font-bold tracking-tight sm:text-4xl"
                                style={jakarta}
                            >
                                Backtest before
                                <br />
                                you bet
                            </h2>
                            <p className="mt-4 text-lg leading-relaxed text-muted-foreground">
                                Replay your strategies against real historical
                                Polymarket order book data. See exactly how they
                                would have performed before risking real
                                capital.
                            </p>
                            <ul className="mt-6 space-y-3">
                                {[
                                    'Real order book data, not simulated prices',
                                    'Full PnL curves, drawdowns, and risk metrics',
                                    'Compare multiple strategies side by side',
                                ].map((item) => (
                                    <li
                                        key={item}
                                        className="flex items-center gap-2.5 text-sm"
                                    >
                                        <Check className="size-4 shrink-0 text-primary" />
                                        <span>{item}</span>
                                    </li>
                                ))}
                            </ul>
                        </div>
                    </div>
                </section>

                {/* ============ FEATURE: Multi-Wallet ============ */}
                <section ref={walletsRef} className="py-24 sm:py-32">
                    <div
                        className={cn(
                            'mx-auto grid max-w-7xl items-center gap-12 px-4 sm:px-6 lg:grid-cols-2 lg:gap-20 lg:px-8',
                            'transition-all duration-700',
                            walletsInView
                                ? 'translate-y-0 opacity-100'
                                : 'translate-y-8 opacity-0',
                        )}
                    >
                        {/* Text */}
                        <div>
                            <p
                                className="text-xs font-semibold tracking-widest text-primary uppercase"
                                style={jakarta}
                            >
                                Multi-Wallet
                            </p>
                            <h2
                                className="mt-3 text-3xl font-bold tracking-tight sm:text-4xl"
                                style={jakarta}
                            >
                                One dashboard,
                                <br />
                                many wallets
                            </h2>
                            <p className="mt-4 text-lg leading-relaxed text-muted-foreground">
                                Generate and manage multiple Polygon wallets
                                from a single interface. Assign different
                                strategies to each wallet and monitor everything
                                in one place.
                            </p>
                            <ul className="mt-6 space-y-3">
                                {[
                                    'Generate unlimited Polygon wallets',
                                    'Assign unique strategies per wallet',
                                    'Unified PnL and performance view',
                                ].map((item) => (
                                    <li
                                        key={item}
                                        className="flex items-center gap-2.5 text-sm"
                                    >
                                        <Check className="size-4 shrink-0 text-primary" />
                                        <span>{item}</span>
                                    </li>
                                ))}
                            </ul>
                        </div>

                        {/* Mock UI */}
                        <div className="rounded-xl border border-border/50 bg-card p-6 shadow-sm">
                            <div className="mb-4 flex items-center gap-2">
                                <div className="size-2.5 rounded-full bg-red-400/60" />
                                <div className="size-2.5 rounded-full bg-amber-400/60" />
                                <div className="size-2.5 rounded-full bg-green-400/60" />
                                <span className="ml-2 text-xs text-muted-foreground">
                                    Wallet Manager
                                </span>
                            </div>
                            <div className="grid grid-cols-2 gap-3">
                                {[
                                    {
                                        name: 'Alpha',
                                        strategies: 3,
                                        active: true,
                                        pnl: '+$412',
                                    },
                                    {
                                        name: 'Beta',
                                        strategies: 1,
                                        active: true,
                                        pnl: '+$89',
                                    },
                                    {
                                        name: 'Hedge',
                                        strategies: 2,
                                        active: true,
                                        pnl: '+$203',
                                    },
                                    {
                                        name: 'Test',
                                        strategies: 1,
                                        active: false,
                                        pnl: '$0',
                                    },
                                ].map((w) => (
                                    <div
                                        key={w.name}
                                        className="rounded-lg border border-border/50 bg-muted/20 p-3"
                                    >
                                        <div className="flex items-center justify-between">
                                            <span className="text-sm font-semibold">
                                                {w.name}
                                            </span>
                                            <span
                                                className={cn(
                                                    'text-[10px] font-medium',
                                                    w.active
                                                        ? 'text-chart-3'
                                                        : 'text-muted-foreground',
                                                )}
                                            >
                                                {w.active
                                                    ? '● Active'
                                                    : '○ Paused'}
                                            </span>
                                        </div>
                                        <div className="mt-2 flex items-center justify-between text-xs text-muted-foreground">
                                            <span>
                                                {w.strategies}{' '}
                                                {w.strategies === 1
                                                    ? 'strategy'
                                                    : 'strategies'}
                                            </span>
                                            <span
                                                className={cn(
                                                    'font-medium',
                                                    w.pnl.startsWith('+') &&
                                                        'text-chart-3',
                                                )}
                                            >
                                                {w.pnl}
                                            </span>
                                        </div>
                                    </div>
                                ))}
                            </div>
                        </div>
                    </div>
                </section>

                {/* ============ SECONDARY FEATURES ============ */}
                <section
                    ref={extrasRef}
                    className="border-y border-border/50 bg-muted/20 py-24 sm:py-32"
                >
                    <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
                        <div className="mx-auto grid gap-px overflow-hidden rounded-xl border border-border/50 bg-border/50 sm:grid-cols-3">
                            {secondaryFeatures.map((feature, i) => (
                                <div
                                    key={feature.title}
                                    className={cn(
                                        'bg-card p-8 transition-all duration-500 hover:bg-primary/[0.02]',
                                        extrasInView
                                            ? 'translate-y-0 opacity-100'
                                            : 'translate-y-8 opacity-0',
                                    )}
                                    style={{
                                        transitionDelay: `${i * 100}ms`,
                                    }}
                                >
                                    <div className="mb-4 inline-flex rounded-lg bg-primary/10 p-2.5 text-primary">
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
                                style={jakarta}
                            >
                                Pricing
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
                                            style={jakarta}
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
                            style={jakarta}
                        >
                            Start trading{' '}
                            <span className="text-primary">smarter</span>
                        </h2>
                        <p className="mx-auto mt-6 max-w-xl text-lg text-muted-foreground">
                            Join traders automating their Polymarket strategies.
                            Free to start, no credit card required.
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
                            <span className="font-bold" style={jakarta}>
                                CraftStrat
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
                            &copy; {new Date().getFullYear()} CraftStrat. All rights
                            reserved.
                        </p>
                    </div>
                </footer>
            </div>
        </>
    );
}
