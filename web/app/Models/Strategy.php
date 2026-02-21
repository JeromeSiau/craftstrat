<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;
use Illuminate\Database\Eloquent\Relations\BelongsToMany;
use Illuminate\Database\Eloquent\Relations\HasMany;

class Strategy extends Model
{
    /** @use HasFactory<\Database\Factories\StrategyFactory> */
    use HasFactory;

    protected $fillable = [
        'user_id',
        'name',
        'description',
        'graph',
        'mode',
        'is_active',
    ];

    protected function casts(): array
    {
        return [
            'graph' => 'array',
            'is_active' => 'boolean',
        ];
    }

    public function user(): BelongsTo
    {
        return $this->belongsTo(User::class);
    }

    public function wallets(): BelongsToMany
    {
        return $this->belongsToMany(Wallet::class, 'wallet_strategies')
            ->using(WalletStrategy::class)
            ->withPivot('markets', 'max_position_usdc', 'is_running', 'started_at');
    }

    public function walletStrategies(): HasMany
    {
        return $this->hasMany(WalletStrategy::class);
    }

    public function trades(): HasMany
    {
        return $this->hasMany(Trade::class);
    }

    public function backtestResults(): HasMany
    {
        return $this->hasMany(BacktestResult::class);
    }
}
