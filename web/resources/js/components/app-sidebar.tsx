import { Link } from '@inertiajs/react';
import { BarChart3, BookOpen, CreditCard, LayoutGrid, LineChart, Target, Wallet } from 'lucide-react';
import { NavFooter } from '@/components/nav-footer';
import { NavMain } from '@/components/nav-main';
import { NavUser } from '@/components/nav-user';
import {
    Sidebar,
    SidebarContent,
    SidebarFooter,
    SidebarHeader,
    SidebarMenu,
    SidebarMenuButton,
    SidebarMenuItem,
} from '@/components/ui/sidebar';
import type { NavItem } from '@/types';
import AppLogo from './app-logo';
import { dashboard } from '@/routes';
import { index as strategiesIndex } from '@/actions/App/Http/Controllers/StrategyController';
import { index as walletsIndex } from '@/actions/App/Http/Controllers/WalletController';
import { index as backtestsIndex } from '@/actions/App/Http/Controllers/BacktestController';
import { index as analyticsIndex } from '@/actions/App/Http/Controllers/AnalyticsController';
import { index as billingIndex } from '@/actions/App/Http/Controllers/BillingController';

const mainNavItems: NavItem[] = [
    { title: 'Dashboard', href: dashboard(), icon: LayoutGrid },
    { title: 'Strategies', href: strategiesIndex.url(), icon: Target },
    { title: 'Wallets', href: walletsIndex.url(), icon: Wallet },
    { title: 'Backtests', href: backtestsIndex.url(), icon: LineChart },
    { title: 'Analytics', href: analyticsIndex.url(), icon: BarChart3 },
    { title: 'Billing', href: billingIndex.url(), icon: CreditCard },
];

const footerNavItems: NavItem[] = [
    { title: 'Documentation', href: 'https://docs.craftstrat.com', icon: BookOpen },
];

export function AppSidebar() {
    return (
        <Sidebar collapsible="icon" variant="inset">
            <SidebarHeader>
                <SidebarMenu>
                    <SidebarMenuItem>
                        <SidebarMenuButton size="lg" asChild>
                            <Link href={dashboard()} prefetch>
                                <AppLogo />
                            </Link>
                        </SidebarMenuButton>
                    </SidebarMenuItem>
                </SidebarMenu>
            </SidebarHeader>

            <SidebarContent>
                <NavMain items={mainNavItems} />
            </SidebarContent>

            <SidebarFooter>
                <NavFooter items={footerNavItems} className="mt-auto" />
                <NavUser />
            </SidebarFooter>
        </Sidebar>
    );
}
