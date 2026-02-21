<?php

namespace App\Models;

// use Illuminate\Contracts\Auth\MustVerifyEmail;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Relations\HasMany;
use Illuminate\Foundation\Auth\User as Authenticatable;
use Illuminate\Notifications\Notifiable;
use Laravel\Cashier\Billable;
use Laravel\Fortify\TwoFactorAuthenticatable;

class User extends Authenticatable
{
    /** @use HasFactory<\Database\Factories\UserFactory> */
    use Billable, HasFactory, Notifiable, TwoFactorAuthenticatable;

    /**
     * The attributes that are mass assignable.
     *
     * @var list<string>
     */
    protected $fillable = [
        'name',
        'email',
        'password',
        'plan',
        'stripe_id',
    ];

    /**
     * The attributes that should be hidden for serialization.
     *
     * @var list<string>
     */
    protected $hidden = [
        'password',
        'two_factor_secret',
        'two_factor_recovery_codes',
        'remember_token',
    ];

    /**
     * Get the attributes that should be cast.
     *
     * @return array<string, string>
     */
    protected function casts(): array
    {
        return [
            'email_verified_at' => 'datetime',
            'password' => 'hashed',
            'two_factor_confirmed_at' => 'datetime',
        ];
    }

    public function strategies(): HasMany
    {
        return $this->hasMany(Strategy::class);
    }

    public function wallets(): HasMany
    {
        return $this->hasMany(Wallet::class);
    }

    public function backtestResults(): HasMany
    {
        return $this->hasMany(BacktestResult::class);
    }

    /**
     * @return array{max_wallets: int|null, max_strategies: int|null, max_leaders: int|null, backtest_days: int|null}
     */
    public function planLimits(): array
    {
        return match ($this->plan ?? 'free') {
            'free' => ['max_wallets' => 1, 'max_strategies' => 2, 'max_leaders' => 1, 'backtest_days' => 30],
            'starter' => ['max_wallets' => 5, 'max_strategies' => 10, 'max_leaders' => 5, 'backtest_days' => null],
            'pro' => ['max_wallets' => 25, 'max_strategies' => null, 'max_leaders' => null, 'backtest_days' => null],
            'enterprise' => ['max_wallets' => null, 'max_strategies' => null, 'max_leaders' => null, 'backtest_days' => null],
            default => ['max_wallets' => 1, 'max_strategies' => 2, 'max_leaders' => 1, 'backtest_days' => 30],
        };
    }
}
