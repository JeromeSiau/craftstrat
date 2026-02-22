<?php

namespace App\Http\Controllers;

use Inertia\Inertia;
use Inertia\Response;

class DashboardController extends Controller
{
    public function index(): Response
    {
        $user = auth()->user();

        return Inertia::render('dashboard', [
            'stats' => [
                'active_strategies' => $user->strategies()->where('is_active', true)->count(),
                'total_strategies' => $user->strategies()->count(),
                'total_wallets' => $user->wallets()->count(),
                'total_pnl_usdc' => $user->wallets()
                    ->join('trades', 'wallets.id', '=', 'trades.wallet_id')
                    ->where('trades.status', 'filled')
                    ->sum('trades.size_usdc'),
                'running_assignments' => $user->wallets()
                    ->join('wallet_strategies', 'wallets.id', '=', 'wallet_strategies.wallet_id')
                    ->where('wallet_strategies.is_running', true)
                    ->count(),
            ],
            'recentStrategies' => $user->strategies()
                ->withCount('wallets')
                ->latest()
                ->limit(5)
                ->get(),
        ]);
    }
}
