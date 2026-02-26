<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;
use Illuminate\Database\Eloquent\Relations\BelongsToMany;
use Illuminate\Database\Eloquent\Relations\HasMany;

class Wallet extends Model
{
    /** @use HasFactory<\Database\Factories\WalletFactory> */
    use HasFactory;

    public $timestamps = false;

    protected $fillable = [
        'user_id',
        'label',
        'signer_address',
        'safe_address',
        'status',
        'deployed_at',
        'balance_usdc',
        'is_active',
    ];

    protected function casts(): array
    {
        return [
            'balance_usdc' => 'decimal:6',
            'is_active' => 'boolean',
            'deployed_at' => 'datetime',
        ];
    }

    /**
     * @var list<string>
     */
    protected $hidden = [
        'private_key_enc',
    ];

    public function isDeployed(): bool
    {
        return $this->status === 'deployed';
    }

    public function isDeploying(): bool
    {
        return in_array($this->status, ['pending', 'deploying']);
    }

    public function isFailed(): bool
    {
        return $this->status === 'failed';
    }

    public function user(): BelongsTo
    {
        return $this->belongsTo(User::class);
    }

    public function strategies(): BelongsToMany
    {
        return $this->belongsToMany(Strategy::class, 'wallet_strategies')
            ->using(WalletStrategy::class)
            ->withPivot('markets', 'max_position_usdc', 'is_running', 'is_paper', 'started_at');
    }

    public function walletStrategies(): HasMany
    {
        return $this->hasMany(WalletStrategy::class);
    }

    public function trades(): HasMany
    {
        return $this->hasMany(Trade::class);
    }

    public function copyRelationships(): HasMany
    {
        return $this->hasMany(CopyRelationship::class, 'follower_wallet_id');
    }
}
