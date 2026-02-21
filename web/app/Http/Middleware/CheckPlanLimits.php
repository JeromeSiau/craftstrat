<?php

namespace App\Http\Middleware;

use App\Models\CopyRelationship;
use Closure;
use Illuminate\Http\Request;
use Symfony\Component\HttpFoundation\Response;

class CheckPlanLimits
{
    /**
     * @param  \Closure(\Illuminate\Http\Request): (\Symfony\Component\HttpFoundation\Response)  $next
     */
    public function handle(Request $request, Closure $next, string $resource): Response
    {
        $user = $request->user();

        if (! $user) {
            return $next($request);
        }

        $limits = $user->planLimits();

        $exceeded = match ($resource) {
            'wallets' => $this->checkLimit($limits['max_wallets'], $user->wallets()->count()),
            'strategies' => $this->checkLimit($limits['max_strategies'], $user->strategies()->count()),
            'leaders' => $this->checkLimit(
                $limits['max_leaders'],
                CopyRelationship::whereIn('follower_wallet_id', $user->wallets()->select('id'))->count(),
            ),
            default => false,
        };

        if ($exceeded) {
            return back()->with('error', "You have reached the maximum number of {$resource} for your plan. Please upgrade.");
        }

        return $next($request);
    }

    private function checkLimit(?int $max, int $current): bool
    {
        if ($max === null) {
            return false;
        }

        return $current >= $max;
    }
}
