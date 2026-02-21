<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;

class BacktestResult extends Model
{
    /** @use HasFactory<\Database\Factories\BacktestResultFactory> */
    use HasFactory;

    public $timestamps = false;

    protected $fillable = [
        'user_id',
        'strategy_id',
        'market_filter',
        'date_from',
        'date_to',
        'total_trades',
        'win_rate',
        'total_pnl_usdc',
        'max_drawdown',
        'sharpe_ratio',
        'result_detail',
    ];

    protected function casts(): array
    {
        return [
            'market_filter' => 'array',
            'date_from' => 'datetime',
            'date_to' => 'datetime',
            'total_trades' => 'integer',
            'win_rate' => 'decimal:4',
            'total_pnl_usdc' => 'decimal:6',
            'max_drawdown' => 'decimal:4',
            'sharpe_ratio' => 'decimal:4',
            'result_detail' => 'array',
        ];
    }

    public function user(): BelongsTo
    {
        return $this->belongsTo(User::class);
    }

    public function strategy(): BelongsTo
    {
        return $this->belongsTo(Strategy::class);
    }
}
