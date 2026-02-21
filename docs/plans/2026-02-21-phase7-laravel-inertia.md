# Phase 7 — Laravel + Inertia Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build the full Laravel domain layer — Models, Services, Controllers, Policies, Middleware, and basic Inertia pages — connecting the existing database schema and Rust engine API to user-facing features.

**Architecture:** Domain models with Eloquent relationships and factories. Controllers return Inertia pages with typed props. WalletService handles Ethereum keypair generation + AES-256-GCM encryption. CheckPlanLimits middleware enforces subscription tier limits. Stripe Cashier manages billing. EngineService (already built) bridges to Rust engine.

**Tech Stack:** Laravel 12, Inertia.js v2, React 19, Pest 4, Stripe Cashier, kornrunner/keccak, simplito/elliptic-php

---

## Task 1: Strategy Model + Factory

**Files:**
- Create: `web/app/Models/Strategy.php`
- Create: `web/database/factories/StrategyFactory.php`
- Create: `web/tests/Unit/Models/StrategyTest.php`

**Step 1: Create the model via artisan**

```bash
cd web && php artisan make:model Strategy --no-interaction
```

**Step 2: Implement the Strategy model**

Replace `app/Models/Strategy.php`:

```php
<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;
use Illuminate\Database\Eloquent\Relations\BelongsToMany;
use Illuminate\Database\Eloquent\Relations\HasMany;

class Strategy extends Model
{
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
```

**Step 3: Create the factory**

```bash
cd web && php artisan make:factory StrategyFactory --no-interaction
```

Replace `database/factories/StrategyFactory.php`:

```php
<?php

namespace Database\Factories;

use App\Models\User;
use Illuminate\Database\Eloquent\Factories\Factory;

class StrategyFactory extends Factory
{
    public function definition(): array
    {
        return [
            'user_id' => User::factory(),
            'name' => fake()->words(3, true),
            'description' => fake()->optional()->sentence(),
            'graph' => [
                'mode' => 'form',
                'conditions' => [
                    [
                        'type' => 'AND',
                        'rules' => [
                            ['indicator' => 'abs_move_pct', 'operator' => '>', 'value' => 3.0],
                        ],
                    ],
                ],
                'action' => [
                    'signal' => 'buy',
                    'outcome' => 'UP',
                    'size_mode' => 'fixed',
                    'size_usdc' => 50,
                    'order_type' => 'market',
                ],
                'risk' => [
                    'stoploss_pct' => 30,
                    'take_profit_pct' => 80,
                    'max_position_usdc' => 200,
                    'max_trades_per_slot' => 1,
                ],
            ],
            'mode' => 'form',
            'is_active' => false,
        ];
    }

    public function active(): static
    {
        return $this->state(fn () => ['is_active' => true]);
    }

    public function nodeMode(): static
    {
        return $this->state(fn () => [
            'mode' => 'node',
            'graph' => [
                'mode' => 'node',
                'nodes' => [
                    ['id' => 'n1', 'type' => 'input', 'data' => ['field' => 'abs_move_pct']],
                    ['id' => 'n2', 'type' => 'comparator', 'data' => ['operator' => '>', 'value' => 3.0]],
                    ['id' => 'n3', 'type' => 'action', 'data' => ['signal' => 'buy', 'outcome' => 'UP', 'size_usdc' => 50]],
                ],
                'edges' => [
                    ['source' => 'n1', 'target' => 'n2'],
                    ['source' => 'n2', 'target' => 'n3'],
                ],
            ],
        ]);
    }
}
```

**Step 4: Write the test**

```bash
cd web && php artisan make:test --pest --unit Models/StrategyTest --no-interaction
```

Replace `tests/Unit/Models/StrategyTest.php`:

```php
<?php

use App\Models\Strategy;
use App\Models\User;
use App\Models\Wallet;
use App\Models\WalletStrategy;

it('belongs to a user', function () {
    $strategy = Strategy::factory()->create();

    expect($strategy->user)->toBeInstanceOf(User::class);
});

it('casts graph as array', function () {
    $strategy = Strategy::factory()->create();

    expect($strategy->graph)->toBeArray()
        ->and($strategy->graph['mode'])->toBe('form');
});

it('casts is_active as boolean', function () {
    $strategy = Strategy::factory()->active()->create();

    expect($strategy->is_active)->toBeTrue();
});

it('has many backtest results', function () {
    $strategy = Strategy::factory()->create();

    expect($strategy->backtestResults)->toBeEmpty();
});
```

**Step 5: Run tests**

```bash
cd web && php artisan test --compact --filter=StrategyTest
```

**Step 6: Commit**

```bash
cd web && vendor/bin/pint --dirty --format agent
git add app/Models/Strategy.php database/factories/StrategyFactory.php tests/Unit/Models/StrategyTest.php
git commit -m "feat(models): add Strategy model with factory and tests"
```

---

## Task 2: Wallet Model + Factory

**Files:**
- Create: `web/app/Models/Wallet.php`
- Create: `web/database/factories/WalletFactory.php`
- Create: `web/tests/Unit/Models/WalletTest.php`

**Step 1: Create the model**

```bash
cd web && php artisan make:model Wallet --no-interaction
```

**Step 2: Implement Wallet model**

```php
<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;
use Illuminate\Database\Eloquent\Relations\BelongsToMany;
use Illuminate\Database\Eloquent\Relations\HasMany;

class Wallet extends Model
{
    use HasFactory;

    public $timestamps = false;

    protected $fillable = [
        'user_id',
        'label',
        'address',
        'private_key_enc',
        'balance_usdc',
        'is_active',
    ];

    protected function casts(): array
    {
        return [
            'balance_usdc' => 'decimal:6',
            'is_active' => 'boolean',
            'private_key_enc' => 'encrypted',
        ];
    }

    protected $hidden = [
        'private_key_enc',
    ];

    public function user(): BelongsTo
    {
        return $this->belongsTo(User::class);
    }

    public function strategies(): BelongsToMany
    {
        return $this->belongsToMany(Strategy::class, 'wallet_strategies')
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

    public function copyRelationships(): HasMany
    {
        return $this->hasMany(CopyRelationship::class, 'follower_wallet_id');
    }
}
```

**Step 3: Create factory**

```bash
cd web && php artisan make:factory WalletFactory --no-interaction
```

```php
<?php

namespace Database\Factories;

use App\Models\User;
use Illuminate\Database\Eloquent\Factories\Factory;

class WalletFactory extends Factory
{
    public function definition(): array
    {
        return [
            'user_id' => User::factory(),
            'label' => fake()->optional()->words(2, true),
            'address' => '0x' . fake()->regexify('[a-fA-F0-9]{40}'),
            'private_key_enc' => encrypt(fake()->sha256()),
            'balance_usdc' => fake()->randomFloat(6, 0, 10000),
            'is_active' => true,
        ];
    }

    public function inactive(): static
    {
        return $this->state(fn () => ['is_active' => false]);
    }
}
```

**Step 4: Write the test**

```bash
cd web && php artisan make:test --pest --unit Models/WalletTest --no-interaction
```

```php
<?php

use App\Models\User;
use App\Models\Wallet;

it('belongs to a user', function () {
    $wallet = Wallet::factory()->create();

    expect($wallet->user)->toBeInstanceOf(User::class);
});

it('hides private_key_enc from serialization', function () {
    $wallet = Wallet::factory()->create();

    expect($wallet->toArray())->not->toHaveKey('private_key_enc');
});

it('casts balance_usdc as decimal', function () {
    $wallet = Wallet::factory()->create(['balance_usdc' => 100.123456]);

    expect($wallet->balance_usdc)->toBe('100.123456');
});

it('casts is_active as boolean', function () {
    $wallet = Wallet::factory()->create();

    expect($wallet->is_active)->toBeTrue();
});
```

**Step 5: Run tests**

```bash
cd web && php artisan test --compact --filter=WalletTest
```

**Step 6: Commit**

```bash
cd web && vendor/bin/pint --dirty --format agent
git add app/Models/Wallet.php database/factories/WalletFactory.php tests/Unit/Models/WalletTest.php
git commit -m "feat(models): add Wallet model with factory and tests"
```

---

## Task 3: Supporting Models — WalletStrategy, Trade, WatchedWallet, CopyRelationship, CopyTrade, BacktestResult

**Files:**
- Create: `web/app/Models/WalletStrategy.php`
- Create: `web/app/Models/Trade.php`
- Create: `web/app/Models/WatchedWallet.php`
- Create: `web/app/Models/CopyRelationship.php`
- Create: `web/app/Models/CopyTrade.php`
- Create: `web/app/Models/BacktestResult.php`
- Create: factories for each
- Create: `web/tests/Unit/Models/SupportingModelsTest.php`

**Step 1: Create all models**

```bash
cd web
php artisan make:model WalletStrategy --no-interaction
php artisan make:model Trade --no-interaction
php artisan make:model WatchedWallet --no-interaction
php artisan make:model CopyRelationship --no-interaction
php artisan make:model CopyTrade --no-interaction
php artisan make:model BacktestResult --no-interaction
```

**Step 2: Implement WalletStrategy (pivot model)**

```php
<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Relations\BelongsTo;
use Illuminate\Database\Eloquent\Relations\Pivot;

class WalletStrategy extends Pivot
{
    protected $table = 'wallet_strategies';

    public $incrementing = true;

    public $timestamps = false;

    protected $fillable = [
        'wallet_id',
        'strategy_id',
        'markets',
        'max_position_usdc',
        'is_running',
        'started_at',
    ];

    protected function casts(): array
    {
        return [
            'markets' => 'array',
            'max_position_usdc' => 'decimal:6',
            'is_running' => 'boolean',
            'started_at' => 'datetime',
        ];
    }

    public function wallet(): BelongsTo
    {
        return $this->belongsTo(Wallet::class);
    }

    public function strategy(): BelongsTo
    {
        return $this->belongsTo(Strategy::class);
    }
}
```

**Step 3: Implement Trade model**

```php
<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;

class Trade extends Model
{
    use HasFactory;

    public $timestamps = false;

    protected $fillable = [
        'wallet_id',
        'strategy_id',
        'copy_relationship_id',
        'market_id',
        'side',
        'outcome',
        'price',
        'size_usdc',
        'order_type',
        'status',
        'polymarket_order_id',
        'fee_bps',
        'executed_at',
    ];

    protected function casts(): array
    {
        return [
            'price' => 'decimal:6',
            'size_usdc' => 'decimal:6',
            'fee_bps' => 'integer',
            'executed_at' => 'datetime',
        ];
    }

    public function wallet(): BelongsTo
    {
        return $this->belongsTo(Wallet::class);
    }

    public function strategy(): BelongsTo
    {
        return $this->belongsTo(Strategy::class);
    }

    public function copyRelationship(): BelongsTo
    {
        return $this->belongsTo(CopyRelationship::class);
    }
}
```

**Step 4: Implement WatchedWallet model**

```php
<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\HasMany;

class WatchedWallet extends Model
{
    use HasFactory;

    public $timestamps = false;

    protected $fillable = [
        'address',
        'label',
        'follower_count',
        'win_rate',
        'total_pnl_usdc',
        'avg_slippage',
        'last_seen_at',
    ];

    protected function casts(): array
    {
        return [
            'follower_count' => 'integer',
            'win_rate' => 'decimal:4',
            'total_pnl_usdc' => 'decimal:6',
            'avg_slippage' => 'decimal:6',
            'last_seen_at' => 'datetime',
        ];
    }

    public function copyRelationships(): HasMany
    {
        return $this->hasMany(CopyRelationship::class);
    }
}
```

**Step 5: Implement CopyRelationship model**

```php
<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;
use Illuminate\Database\Eloquent\Relations\HasMany;

class CopyRelationship extends Model
{
    use HasFactory;

    public $timestamps = false;

    protected $fillable = [
        'follower_wallet_id',
        'watched_wallet_id',
        'size_mode',
        'size_value',
        'max_position_usdc',
        'markets_filter',
        'is_active',
    ];

    protected function casts(): array
    {
        return [
            'size_value' => 'decimal:6',
            'max_position_usdc' => 'decimal:6',
            'markets_filter' => 'array',
            'is_active' => 'boolean',
        ];
    }

    public function followerWallet(): BelongsTo
    {
        return $this->belongsTo(Wallet::class, 'follower_wallet_id');
    }

    public function watchedWallet(): BelongsTo
    {
        return $this->belongsTo(WatchedWallet::class);
    }

    public function copyTrades(): HasMany
    {
        return $this->hasMany(CopyTrade::class);
    }
}
```

**Step 6: Implement CopyTrade model**

```php
<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;

class CopyTrade extends Model
{
    use HasFactory;

    public $timestamps = false;

    protected $fillable = [
        'copy_relationship_id',
        'follower_trade_id',
        'leader_address',
        'leader_market_id',
        'leader_outcome',
        'leader_price',
        'leader_size_usdc',
        'leader_tx_hash',
        'follower_price',
        'slippage_pct',
        'status',
        'skip_reason',
        'detected_at',
        'executed_at',
    ];

    protected function casts(): array
    {
        return [
            'leader_price' => 'decimal:6',
            'leader_size_usdc' => 'decimal:6',
            'follower_price' => 'decimal:6',
            'slippage_pct' => 'decimal:6',
            'detected_at' => 'datetime',
            'executed_at' => 'datetime',
        ];
    }

    public function copyRelationship(): BelongsTo
    {
        return $this->belongsTo(CopyRelationship::class);
    }

    public function followerTrade(): BelongsTo
    {
        return $this->belongsTo(Trade::class, 'follower_trade_id');
    }
}
```

**Step 7: Implement BacktestResult model**

```php
<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;

class BacktestResult extends Model
{
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
```

**Step 8: Create all factories**

```bash
cd web
php artisan make:factory TradeFactory --no-interaction
php artisan make:factory WatchedWalletFactory --no-interaction
php artisan make:factory CopyRelationshipFactory --no-interaction
php artisan make:factory CopyTradeFactory --no-interaction
php artisan make:factory BacktestResultFactory --no-interaction
```

TradeFactory:
```php
<?php

namespace Database\Factories;

use App\Models\Wallet;
use App\Models\Strategy;
use Illuminate\Database\Eloquent\Factories\Factory;

class TradeFactory extends Factory
{
    public function definition(): array
    {
        return [
            'wallet_id' => Wallet::factory(),
            'strategy_id' => Strategy::factory(),
            'market_id' => 'btc-updown-15m-' . fake()->unixTime(),
            'side' => fake()->randomElement(['buy', 'sell']),
            'outcome' => fake()->randomElement(['UP', 'DOWN']),
            'price' => fake()->randomFloat(6, 0.01, 0.99),
            'size_usdc' => fake()->randomFloat(6, 10, 500),
            'order_type' => fake()->randomElement(['market', 'limit']),
            'status' => 'filled',
            'fee_bps' => fake()->randomElement([0, 50, 100]),
            'executed_at' => now(),
        ];
    }

    public function pending(): static
    {
        return $this->state(fn () => ['status' => 'pending', 'executed_at' => null]);
    }
}
```

WatchedWalletFactory:
```php
<?php

namespace Database\Factories;

use Illuminate\Database\Eloquent\Factories\Factory;

class WatchedWalletFactory extends Factory
{
    public function definition(): array
    {
        return [
            'address' => '0x' . fake()->regexify('[a-fA-F0-9]{40}'),
            'label' => fake()->optional()->words(2, true),
            'follower_count' => fake()->numberBetween(0, 100),
            'win_rate' => fake()->optional()->randomFloat(4, 0.3, 0.8),
            'total_pnl_usdc' => fake()->optional()->randomFloat(6, -1000, 10000),
            'avg_slippage' => fake()->optional()->randomFloat(6, 0, 0.05),
            'last_seen_at' => fake()->optional()->dateTimeThisMonth(),
        ];
    }
}
```

CopyRelationshipFactory:
```php
<?php

namespace Database\Factories;

use App\Models\Wallet;
use App\Models\WatchedWallet;
use Illuminate\Database\Eloquent\Factories\Factory;

class CopyRelationshipFactory extends Factory
{
    public function definition(): array
    {
        return [
            'follower_wallet_id' => Wallet::factory(),
            'watched_wallet_id' => WatchedWallet::factory(),
            'size_mode' => fake()->randomElement(['fixed', 'proportional']),
            'size_value' => fake()->randomFloat(6, 10, 500),
            'max_position_usdc' => 100,
            'markets_filter' => null,
            'is_active' => true,
        ];
    }
}
```

CopyTradeFactory:
```php
<?php

namespace Database\Factories;

use App\Models\CopyRelationship;
use Illuminate\Database\Eloquent\Factories\Factory;

class CopyTradeFactory extends Factory
{
    public function definition(): array
    {
        return [
            'copy_relationship_id' => CopyRelationship::factory(),
            'leader_address' => '0x' . fake()->regexify('[a-fA-F0-9]{40}'),
            'leader_market_id' => 'btc-updown-15m-' . fake()->unixTime(),
            'leader_outcome' => fake()->randomElement(['UP', 'DOWN']),
            'leader_price' => fake()->randomFloat(6, 0.01, 0.99),
            'leader_size_usdc' => fake()->randomFloat(6, 10, 500),
            'leader_tx_hash' => '0x' . fake()->sha256(),
            'follower_price' => fake()->randomFloat(6, 0.01, 0.99),
            'slippage_pct' => fake()->randomFloat(6, -0.05, 0.05),
            'status' => 'filled',
            'detected_at' => now(),
            'executed_at' => now(),
        ];
    }
}
```

BacktestResultFactory:
```php
<?php

namespace Database\Factories;

use App\Models\Strategy;
use App\Models\User;
use Illuminate\Database\Eloquent\Factories\Factory;

class BacktestResultFactory extends Factory
{
    public function definition(): array
    {
        return [
            'user_id' => User::factory(),
            'strategy_id' => Strategy::factory(),
            'market_filter' => null,
            'date_from' => now()->subDays(30),
            'date_to' => now(),
            'total_trades' => fake()->numberBetween(10, 500),
            'win_rate' => fake()->randomFloat(4, 0.3, 0.8),
            'total_pnl_usdc' => fake()->randomFloat(6, -500, 5000),
            'max_drawdown' => fake()->randomFloat(4, 0.01, 0.5),
            'sharpe_ratio' => fake()->randomFloat(4, -1, 3),
            'result_detail' => null,
        ];
    }
}
```

**Step 9: Write tests**

```bash
cd web && php artisan make:test --pest --unit Models/SupportingModelsTest --no-interaction
```

```php
<?php

use App\Models\BacktestResult;
use App\Models\CopyRelationship;
use App\Models\CopyTrade;
use App\Models\Strategy;
use App\Models\Trade;
use App\Models\User;
use App\Models\Wallet;
use App\Models\WatchedWallet;

it('creates a trade belonging to a wallet and strategy', function () {
    $trade = Trade::factory()->create();

    expect($trade->wallet)->toBeInstanceOf(Wallet::class)
        ->and($trade->strategy)->toBeInstanceOf(Strategy::class);
});

it('creates a watched wallet with relationships', function () {
    $watched = WatchedWallet::factory()->create();
    $relationship = CopyRelationship::factory()->create(['watched_wallet_id' => $watched->id]);

    expect($watched->copyRelationships)->toHaveCount(1);
});

it('creates a copy relationship linking follower to leader', function () {
    $relationship = CopyRelationship::factory()->create();

    expect($relationship->followerWallet)->toBeInstanceOf(Wallet::class)
        ->and($relationship->watchedWallet)->toBeInstanceOf(WatchedWallet::class);
});

it('creates a copy trade linked to a relationship', function () {
    $copyTrade = CopyTrade::factory()->create();

    expect($copyTrade->copyRelationship)->toBeInstanceOf(CopyRelationship::class);
});

it('creates a backtest result belonging to user and strategy', function () {
    $result = BacktestResult::factory()->create();

    expect($result->user)->toBeInstanceOf(User::class)
        ->and($result->strategy)->toBeInstanceOf(Strategy::class)
        ->and($result->result_detail)->toBeNull();
});

it('casts backtest result fields correctly', function () {
    $result = BacktestResult::factory()->create([
        'total_trades' => 42,
        'win_rate' => 0.6523,
        'result_detail' => ['trades' => []],
    ]);

    expect($result->total_trades)->toBe(42)
        ->and($result->win_rate)->toBe('0.6523')
        ->and($result->result_detail)->toBeArray();
});
```

**Step 10: Run tests**

```bash
cd web && php artisan test --compact --filter=SupportingModelsTest
```

**Step 11: Commit**

```bash
cd web && vendor/bin/pint --dirty --format agent
git add app/Models/ database/factories/ tests/Unit/Models/SupportingModelsTest.php
git commit -m "feat(models): add Trade, WatchedWallet, CopyRelationship, CopyTrade, BacktestResult models with factories"
```

---

## Task 4: User Model — Relationships + Plan Helpers

**Files:**
- Modify: `web/app/Models/User.php`
- Create: `web/tests/Unit/Models/UserRelationshipsTest.php`

**Step 1: Add relationships and plan helpers to User model**

Add to `fillable`:
```php
protected $fillable = [
    'name',
    'email',
    'password',
    'plan',
    'stripe_id',
];
```

Add casts for `plan`:
```php
protected function casts(): array
{
    return [
        'email_verified_at' => 'datetime',
        'password' => 'hashed',
        'two_factor_confirmed_at' => 'datetime',
    ];
}
```

Add relationships and plan helpers after the `casts()` method:

```php
use Illuminate\Database\Eloquent\Relations\HasMany;

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
```

**Step 2: Write tests**

```bash
cd web && php artisan make:test --pest --unit Models/UserRelationshipsTest --no-interaction
```

```php
<?php

use App\Models\BacktestResult;
use App\Models\Strategy;
use App\Models\User;
use App\Models\Wallet;

it('has many strategies', function () {
    $user = User::factory()->create();
    Strategy::factory()->count(3)->create(['user_id' => $user->id]);

    expect($user->strategies)->toHaveCount(3);
});

it('has many wallets', function () {
    $user = User::factory()->create();
    Wallet::factory()->count(2)->create(['user_id' => $user->id]);

    expect($user->wallets)->toHaveCount(2);
});

it('has many backtest results', function () {
    $user = User::factory()->create();
    $strategy = Strategy::factory()->create(['user_id' => $user->id]);
    BacktestResult::factory()->create(['user_id' => $user->id, 'strategy_id' => $strategy->id]);

    expect($user->backtestResults)->toHaveCount(1);
});

it('returns correct plan limits for free plan', function () {
    $user = User::factory()->create(['plan' => 'free']);
    $limits = $user->planLimits();

    expect($limits['max_wallets'])->toBe(1)
        ->and($limits['max_strategies'])->toBe(2)
        ->and($limits['max_leaders'])->toBe(1)
        ->and($limits['backtest_days'])->toBe(30);
});

it('returns correct plan limits for pro plan', function () {
    $user = User::factory()->create(['plan' => 'pro']);
    $limits = $user->planLimits();

    expect($limits['max_wallets'])->toBe(25)
        ->and($limits['max_strategies'])->toBeNull()
        ->and($limits['max_leaders'])->toBeNull()
        ->and($limits['backtest_days'])->toBeNull();
});

it('returns correct plan limits for enterprise plan', function () {
    $user = User::factory()->create(['plan' => 'enterprise']);
    $limits = $user->planLimits();

    expect($limits['max_wallets'])->toBeNull()
        ->and($limits['max_strategies'])->toBeNull();
});

it('defaults to free plan limits when plan is null', function () {
    $user = User::factory()->create(['plan' => null]);
    $limits = $user->planLimits();

    expect($limits['max_wallets'])->toBe(1);
});
```

**Step 3: Run tests**

```bash
cd web && php artisan test --compact --filter=UserRelationshipsTest
```

**Step 4: Commit**

```bash
cd web && vendor/bin/pint --dirty --format agent
git add app/Models/User.php tests/Unit/Models/UserRelationshipsTest.php
git commit -m "feat(models): add User relationships and plan limit helpers"
```

---

## Task 5: WalletService — Keypair Generation + AES-256-GCM Encryption

**Files:**
- Create: `web/app/Services/WalletService.php`
- Modify: `web/app/Providers/AppServiceProvider.php`
- Modify: `web/config/services.php`
- Create: `web/tests/Unit/Services/WalletServiceTest.php`

**Step 1: Install required packages**

```bash
cd web && composer require kornrunner/keccak simplito/elliptic-php --no-interaction
```

**Step 2: Add wallet encryption key to config/services.php**

Add to the `services.php` array:
```php
'wallet' => [
    'encryption_key' => env('ENCRYPTION_KEY'),
],
```

**Step 3: Implement WalletService**

```php
<?php

namespace App\Services;

use Elliptic\EC;
use kornrunner\Keccak;
use RuntimeException;

class WalletService
{
    private readonly string $encryptionKey;

    public function __construct(string $encryptionKey)
    {
        if (strlen($encryptionKey) < 32) {
            throw new RuntimeException('ENCRYPTION_KEY must be at least 32 characters.');
        }

        $this->encryptionKey = $encryptionKey;
    }

    /**
     * Generate a new Ethereum/Polygon keypair.
     *
     * @return array{address: string, private_key_enc: string}
     */
    public function generateKeypair(): array
    {
        $ec = new EC('secp256k1');
        $keyPair = $ec->genKeyPair();

        $privateKeyHex = $keyPair->getPrivate('hex');
        $publicKeyHex = substr($keyPair->getPublic(false, 'hex'), 2); // remove '04' prefix

        $address = $this->publicKeyToAddress($publicKeyHex);
        $encryptedKey = $this->encrypt($privateKeyHex);

        return [
            'address' => $address,
            'private_key_enc' => $encryptedKey,
        ];
    }

    /**
     * Encrypt a private key with AES-256-GCM.
     */
    public function encrypt(string $plaintext): string
    {
        $key = substr(hash('sha256', $this->encryptionKey, true), 0, 32);
        $iv = random_bytes(12);
        $tag = '';

        $ciphertext = openssl_encrypt(
            $plaintext,
            'aes-256-gcm',
            $key,
            OPENSSL_RAW_DATA,
            $iv,
            $tag,
        );

        if ($ciphertext === false) {
            throw new RuntimeException('Encryption failed.');
        }

        return base64_encode($iv . $tag . $ciphertext);
    }

    /**
     * Decrypt a private key from AES-256-GCM.
     */
    public function decrypt(string $encrypted): string
    {
        $key = substr(hash('sha256', $this->encryptionKey, true), 0, 32);
        $decoded = base64_decode($encrypted, true);

        if ($decoded === false || strlen($decoded) < 28) {
            throw new RuntimeException('Invalid encrypted data.');
        }

        $iv = substr($decoded, 0, 12);
        $tag = substr($decoded, 12, 16);
        $ciphertext = substr($decoded, 28);

        $plaintext = openssl_decrypt(
            $ciphertext,
            'aes-256-gcm',
            $key,
            OPENSSL_RAW_DATA,
            $iv,
            $tag,
        );

        if ($plaintext === false) {
            throw new RuntimeException('Decryption failed — invalid key or corrupted data.');
        }

        return $plaintext;
    }

    /**
     * Derive an EIP-55 checksummed Ethereum address from an uncompressed public key (without 04 prefix).
     */
    private function publicKeyToAddress(string $publicKeyHex): string
    {
        $hash = Keccak::hash(hex2bin($publicKeyHex), 256);
        $addressLower = substr($hash, -40);

        return $this->toChecksumAddress($addressLower);
    }

    /**
     * Apply EIP-55 mixed-case checksum to an address.
     */
    private function toChecksumAddress(string $addressLower): string
    {
        $hash = Keccak::hash($addressLower, 256);
        $checksummed = '0x';

        for ($i = 0; $i < 40; $i++) {
            if (intval($hash[$i], 16) >= 8) {
                $checksummed .= strtoupper($addressLower[$i]);
            } else {
                $checksummed .= $addressLower[$i];
            }
        }

        return $checksummed;
    }
}
```

**Step 4: Register WalletService as singleton in AppServiceProvider**

Add to the `register()` method in `app/Providers/AppServiceProvider.php`:

```php
$this->app->singleton(\App\Services\WalletService::class, function ($app) {
    return new \App\Services\WalletService(
        encryptionKey: config('services.wallet.encryption_key'),
    );
});
```

**Step 5: Write tests**

```bash
cd web && php artisan make:test --pest --unit Services/WalletServiceTest --no-interaction
```

```php
<?php

use App\Services\WalletService;

beforeEach(function () {
    $this->service = new WalletService(
        encryptionKey: 'test-encryption-key-at-least-32-chars-long',
    );
});

it('generates a valid ethereum keypair', function () {
    $result = $this->service->generateKeypair();

    expect($result)->toHaveKeys(['address', 'private_key_enc'])
        ->and($result['address'])->toStartWith('0x')
        ->and($result['address'])->toHaveLength(42)
        ->and($result['private_key_enc'])->not->toBeEmpty();
});

it('generates unique addresses on each call', function () {
    $first = $this->service->generateKeypair();
    $second = $this->service->generateKeypair();

    expect($first['address'])->not->toBe($second['address']);
});

it('encrypts and decrypts a private key correctly', function () {
    $original = 'aabbccddee11223344556677889900aabbccddee11223344556677889900aabb';

    $encrypted = $this->service->encrypt($original);
    $decrypted = $this->service->decrypt($encrypted);

    expect($decrypted)->toBe($original)
        ->and($encrypted)->not->toBe($original);
});

it('can decrypt keys from generated keypair', function () {
    $keypair = $this->service->generateKeypair();
    $decryptedKey = $this->service->decrypt($keypair['private_key_enc']);

    expect($decryptedKey)->toHaveLength(64) // 32 bytes in hex
        ->and($decryptedKey)->toMatch('/^[a-f0-9]{64}$/');
});

it('throws on decryption with wrong key', function () {
    $encrypted = $this->service->encrypt('secret');

    $wrongService = new WalletService(
        encryptionKey: 'different-encryption-key-also-32-chars-long!!',
    );

    $wrongService->decrypt($encrypted);
})->throws(RuntimeException::class, 'Decryption failed');

it('throws when encryption key is too short', function () {
    new WalletService(encryptionKey: 'short');
})->throws(RuntimeException::class, 'ENCRYPTION_KEY must be at least 32 characters');
```

**Step 6: Run tests**

```bash
cd web && php artisan test --compact --filter=WalletServiceTest
```

**Step 7: Commit**

```bash
cd web && vendor/bin/pint --dirty --format agent
git add app/Services/WalletService.php app/Providers/AppServiceProvider.php config/services.php tests/Unit/Services/WalletServiceTest.php composer.json composer.lock
git commit -m "feat(services): add WalletService with Ethereum keypair generation and AES-256-GCM encryption"
```

---

## Task 6: Authorization Policies — Strategy, Wallet, BacktestResult

**Files:**
- Create: `web/app/Policies/StrategyPolicy.php`
- Create: `web/app/Policies/WalletPolicy.php`
- Create: `web/app/Policies/BacktestResultPolicy.php`
- Create: `web/tests/Unit/Policies/StrategyPolicyTest.php`
- Create: `web/tests/Unit/Policies/WalletPolicyTest.php`

**Step 1: Create policies**

```bash
cd web
php artisan make:policy StrategyPolicy --model=Strategy --no-interaction
php artisan make:policy WalletPolicy --model=Wallet --no-interaction
php artisan make:policy BacktestResultPolicy --model=BacktestResult --no-interaction
```

**Step 2: Implement StrategyPolicy**

```php
<?php

namespace App\Policies;

use App\Models\Strategy;
use App\Models\User;

class StrategyPolicy
{
    public function view(User $user, Strategy $strategy): bool
    {
        return $user->id === $strategy->user_id;
    }

    public function update(User $user, Strategy $strategy): bool
    {
        return $user->id === $strategy->user_id;
    }

    public function delete(User $user, Strategy $strategy): bool
    {
        return $user->id === $strategy->user_id;
    }
}
```

**Step 3: Implement WalletPolicy**

```php
<?php

namespace App\Policies;

use App\Models\User;
use App\Models\Wallet;

class WalletPolicy
{
    public function view(User $user, Wallet $wallet): bool
    {
        return $user->id === $wallet->user_id;
    }

    public function delete(User $user, Wallet $wallet): bool
    {
        return $user->id === $wallet->user_id;
    }
}
```

**Step 4: Implement BacktestResultPolicy**

```php
<?php

namespace App\Policies;

use App\Models\BacktestResult;
use App\Models\User;

class BacktestResultPolicy
{
    public function view(User $user, BacktestResult $backtestResult): bool
    {
        return $user->id === $backtestResult->user_id;
    }
}
```

**Step 5: Write tests**

```bash
cd web && php artisan make:test --pest --unit Policies/StrategyPolicyTest --no-interaction
cd web && php artisan make:test --pest --unit Policies/WalletPolicyTest --no-interaction
```

StrategyPolicyTest:
```php
<?php

use App\Models\Strategy;
use App\Models\User;

it('allows owner to view their strategy', function () {
    $user = User::factory()->create();
    $strategy = Strategy::factory()->create(['user_id' => $user->id]);

    expect($user->can('view', $strategy))->toBeTrue();
});

it('prevents user from viewing another users strategy', function () {
    $owner = User::factory()->create();
    $other = User::factory()->create();
    $strategy = Strategy::factory()->create(['user_id' => $owner->id]);

    expect($other->can('view', $strategy))->toBeFalse();
});

it('allows owner to update their strategy', function () {
    $user = User::factory()->create();
    $strategy = Strategy::factory()->create(['user_id' => $user->id]);

    expect($user->can('update', $strategy))->toBeTrue();
});

it('allows owner to delete their strategy', function () {
    $user = User::factory()->create();
    $strategy = Strategy::factory()->create(['user_id' => $user->id]);

    expect($user->can('delete', $strategy))->toBeTrue();
});
```

WalletPolicyTest:
```php
<?php

use App\Models\User;
use App\Models\Wallet;

it('allows owner to view their wallet', function () {
    $user = User::factory()->create();
    $wallet = Wallet::factory()->create(['user_id' => $user->id]);

    expect($user->can('view', $wallet))->toBeTrue();
});

it('prevents user from viewing another users wallet', function () {
    $owner = User::factory()->create();
    $other = User::factory()->create();
    $wallet = Wallet::factory()->create(['user_id' => $owner->id]);

    expect($other->can('view', $wallet))->toBeFalse();
});

it('allows owner to delete their wallet', function () {
    $user = User::factory()->create();
    $wallet = Wallet::factory()->create(['user_id' => $user->id]);

    expect($user->can('delete', $wallet))->toBeTrue();
});
```

**Step 6: Run tests**

```bash
cd web && php artisan test --compact --filter=PolicyTest
```

**Step 7: Commit**

```bash
cd web && vendor/bin/pint --dirty --format agent
git add app/Policies/ tests/Unit/Policies/
git commit -m "feat(policies): add Strategy, Wallet, BacktestResult authorization policies with tests"
```

---

## Task 7: Form Requests

**Files:**
- Create: `web/app/Http/Requests/StoreStrategyRequest.php`
- Create: `web/app/Http/Requests/UpdateStrategyRequest.php`
- Create: `web/app/Http/Requests/StoreWalletRequest.php`
- Create: `web/app/Http/Requests/AssignStrategyRequest.php`
- Create: `web/app/Http/Requests/RunBacktestRequest.php`

**Step 1: Create all form requests**

```bash
cd web
php artisan make:request StoreStrategyRequest --no-interaction
php artisan make:request UpdateStrategyRequest --no-interaction
php artisan make:request StoreWalletRequest --no-interaction
php artisan make:request AssignStrategyRequest --no-interaction
php artisan make:request RunBacktestRequest --no-interaction
```

**Step 2: Implement StoreStrategyRequest**

```php
<?php

namespace App\Http\Requests;

use Illuminate\Foundation\Http\FormRequest;

class StoreStrategyRequest extends FormRequest
{
    public function authorize(): bool
    {
        return true;
    }

    /**
     * @return array<string, \Illuminate\Contracts\Validation\ValidationRule|array<mixed>|string>
     */
    public function rules(): array
    {
        return [
            'name' => ['required', 'string', 'max:255'],
            'description' => ['nullable', 'string'],
            'graph' => ['required', 'array'],
            'graph.mode' => ['required', 'in:form,node'],
            'mode' => ['required', 'in:form,node'],
        ];
    }
}
```

**Step 3: Implement UpdateStrategyRequest**

```php
<?php

namespace App\Http\Requests;

use Illuminate\Foundation\Http\FormRequest;

class UpdateStrategyRequest extends FormRequest
{
    public function authorize(): bool
    {
        return $this->user()->can('update', $this->route('strategy'));
    }

    /**
     * @return array<string, \Illuminate\Contracts\Validation\ValidationRule|array<mixed>|string>
     */
    public function rules(): array
    {
        return [
            'name' => ['sometimes', 'required', 'string', 'max:255'],
            'description' => ['nullable', 'string'],
            'graph' => ['sometimes', 'required', 'array'],
            'graph.mode' => ['required_with:graph', 'in:form,node'],
            'mode' => ['sometimes', 'required', 'in:form,node'],
        ];
    }
}
```

**Step 4: Implement StoreWalletRequest**

```php
<?php

namespace App\Http\Requests;

use Illuminate\Foundation\Http\FormRequest;

class StoreWalletRequest extends FormRequest
{
    public function authorize(): bool
    {
        return true;
    }

    /**
     * @return array<string, \Illuminate\Contracts\Validation\ValidationRule|array<mixed>|string>
     */
    public function rules(): array
    {
        return [
            'label' => ['nullable', 'string', 'max:255'],
        ];
    }
}
```

**Step 5: Implement AssignStrategyRequest**

```php
<?php

namespace App\Http\Requests;

use Illuminate\Foundation\Http\FormRequest;

class AssignStrategyRequest extends FormRequest
{
    public function authorize(): bool
    {
        return true;
    }

    /**
     * @return array<string, \Illuminate\Contracts\Validation\ValidationRule|array<mixed>|string>
     */
    public function rules(): array
    {
        return [
            'strategy_id' => ['required', 'exists:strategies,id'],
            'markets' => ['nullable', 'array'],
            'markets.*' => ['string'],
            'max_position_usdc' => ['nullable', 'numeric', 'min:1', 'max:1000000'],
        ];
    }
}
```

**Step 6: Implement RunBacktestRequest**

```php
<?php

namespace App\Http\Requests;

use Illuminate\Foundation\Http\FormRequest;

class RunBacktestRequest extends FormRequest
{
    public function authorize(): bool
    {
        return $this->user()->can('view', $this->route('strategy'));
    }

    /**
     * @return array<string, \Illuminate\Contracts\Validation\ValidationRule|array<mixed>|string>
     */
    public function rules(): array
    {
        return [
            'market_filter' => ['nullable', 'array'],
            'market_filter.*' => ['string'],
            'date_from' => ['required', 'date'],
            'date_to' => ['required', 'date', 'after:date_from'],
        ];
    }
}
```

**Step 7: Commit**

```bash
cd web && vendor/bin/pint --dirty --format agent
git add app/Http/Requests/
git commit -m "feat(requests): add form request validation for strategies, wallets, and backtests"
```

---

## Task 8: StrategyController + Routes

**Files:**
- Create: `web/app/Http/Controllers/StrategyController.php`
- Modify: `web/routes/web.php`
- Create: `web/tests/Feature/StrategyControllerTest.php`

**Step 1: Create controller**

```bash
cd web && php artisan make:controller StrategyController --no-interaction
```

**Step 2: Implement StrategyController**

```php
<?php

namespace App\Http\Controllers;

use App\Http\Requests\StoreStrategyRequest;
use App\Http\Requests\UpdateStrategyRequest;
use App\Models\Strategy;
use App\Services\EngineService;
use Illuminate\Http\RedirectResponse;
use Inertia\Inertia;
use Inertia\Response;

class StrategyController extends Controller
{
    public function index(): Response
    {
        return Inertia::render('strategies/index', [
            'strategies' => auth()->user()->strategies()
                ->withCount('wallets')
                ->latest()
                ->get(),
        ]);
    }

    public function create(): Response
    {
        return Inertia::render('strategies/create');
    }

    public function store(StoreStrategyRequest $request): RedirectResponse
    {
        $request->user()->strategies()->create($request->validated());

        return to_route('strategies.index')->with('success', 'Strategy created.');
    }

    public function show(Strategy $strategy): Response
    {
        $this->authorize('view', $strategy);

        $strategy->load(['walletStrategies.wallet', 'backtestResults' => fn ($q) => $q->latest()->limit(5)]);

        return Inertia::render('strategies/show', [
            'strategy' => $strategy,
        ]);
    }

    public function update(UpdateStrategyRequest $request, Strategy $strategy): RedirectResponse
    {
        $strategy->update($request->validated());

        return back()->with('success', 'Strategy updated.');
    }

    public function destroy(Strategy $strategy): RedirectResponse
    {
        $this->authorize('delete', $strategy);

        $strategy->delete();

        return to_route('strategies.index')->with('success', 'Strategy deleted.');
    }

    public function activate(Strategy $strategy, EngineService $engine): RedirectResponse
    {
        $this->authorize('update', $strategy);

        $runningAssignments = $strategy->walletStrategies()->where('is_running', false)->with('wallet')->get();

        foreach ($runningAssignments as $assignment) {
            $engine->activateStrategy(
                $assignment->wallet_id,
                $strategy->id,
                $strategy->graph,
                $assignment->markets ?? [],
                (float) $assignment->max_position_usdc,
            );

            $assignment->update(['is_running' => true, 'started_at' => now()]);
        }

        $strategy->update(['is_active' => true]);

        return back()->with('success', 'Strategy activated.');
    }

    public function deactivate(Strategy $strategy, EngineService $engine): RedirectResponse
    {
        $this->authorize('update', $strategy);

        $runningAssignments = $strategy->walletStrategies()->where('is_running', true)->get();

        foreach ($runningAssignments as $assignment) {
            $engine->deactivateStrategy($assignment->wallet_id, $strategy->id);
            $assignment->update(['is_running' => false, 'started_at' => null]);
        }

        $strategy->update(['is_active' => false]);

        return back()->with('success', 'Strategy deactivated.');
    }
}
```

**Step 3: Add routes to web.php**

Add inside the auth middleware group in `routes/web.php`:

```php
use App\Http\Controllers\StrategyController;

Route::middleware(['auth', 'verified'])->group(function () {
    Route::get('dashboard', function () {
        return Inertia::render('dashboard');
    })->name('dashboard');

    // Strategies
    Route::resource('strategies', StrategyController::class)->except(['edit']);
    Route::post('strategies/{strategy}/activate', [StrategyController::class, 'activate'])->name('strategies.activate');
    Route::post('strategies/{strategy}/deactivate', [StrategyController::class, 'deactivate'])->name('strategies.deactivate');
});
```

Note: Remove the existing standalone dashboard route and place it inside this group.

**Step 4: Write feature tests**

```bash
cd web && php artisan make:test --pest StrategyControllerTest --no-interaction
```

```php
<?php

use App\Models\Strategy;
use App\Models\User;
use Illuminate\Support\Facades\Http;
use Inertia\Testing\AssertableInertia as Assert;

beforeEach(function () {
    $this->user = User::factory()->create();
});

it('displays strategies index page', function () {
    Strategy::factory()->count(3)->create(['user_id' => $this->user->id]);

    $this->actingAs($this->user)
        ->get(route('strategies.index'))
        ->assertOk()
        ->assertInertia(fn (Assert $page) => $page
            ->component('strategies/index')
            ->has('strategies', 3)
        );
});

it('displays create strategy page', function () {
    $this->actingAs($this->user)
        ->get(route('strategies.create'))
        ->assertOk()
        ->assertInertia(fn (Assert $page) => $page
            ->component('strategies/create')
        );
});

it('stores a new strategy', function () {
    $this->actingAs($this->user)
        ->post(route('strategies.store'), [
            'name' => 'My Strategy',
            'description' => 'A test strategy',
            'mode' => 'form',
            'graph' => [
                'mode' => 'form',
                'conditions' => [],
                'action' => ['signal' => 'buy', 'outcome' => 'UP', 'size_usdc' => 50, 'size_mode' => 'fixed', 'order_type' => 'market'],
                'risk' => ['max_trades_per_slot' => 1],
            ],
        ])
        ->assertRedirect(route('strategies.index'));

    expect(Strategy::where('user_id', $this->user->id)->count())->toBe(1);
});

it('validates required fields on store', function () {
    $this->actingAs($this->user)
        ->post(route('strategies.store'), [])
        ->assertSessionHasErrors(['name', 'graph', 'mode']);
});

it('shows a strategy belonging to the user', function () {
    $strategy = Strategy::factory()->create(['user_id' => $this->user->id]);

    $this->actingAs($this->user)
        ->get(route('strategies.show', $strategy))
        ->assertOk()
        ->assertInertia(fn (Assert $page) => $page
            ->component('strategies/show')
            ->has('strategy')
        );
});

it('prevents viewing another users strategy', function () {
    $other = User::factory()->create();
    $strategy = Strategy::factory()->create(['user_id' => $other->id]);

    $this->actingAs($this->user)
        ->get(route('strategies.show', $strategy))
        ->assertForbidden();
});

it('updates a strategy', function () {
    $strategy = Strategy::factory()->create(['user_id' => $this->user->id]);

    $this->actingAs($this->user)
        ->put(route('strategies.update', $strategy), ['name' => 'Updated Name'])
        ->assertRedirect();

    expect($strategy->fresh()->name)->toBe('Updated Name');
});

it('deletes a strategy', function () {
    $strategy = Strategy::factory()->create(['user_id' => $this->user->id]);

    $this->actingAs($this->user)
        ->delete(route('strategies.destroy', $strategy))
        ->assertRedirect(route('strategies.index'));

    expect(Strategy::find($strategy->id))->toBeNull();
});

it('activates a strategy and calls engine', function () {
    Http::fake(['*/internal/strategy/activate' => Http::response(null, 200)]);

    $strategy = Strategy::factory()->create(['user_id' => $this->user->id]);

    $this->actingAs($this->user)
        ->post(route('strategies.activate', $strategy))
        ->assertRedirect();

    expect($strategy->fresh()->is_active)->toBeTrue();
});

it('deactivates a strategy and calls engine', function () {
    Http::fake(['*/internal/strategy/deactivate' => Http::response(null, 200)]);

    $strategy = Strategy::factory()->active()->create(['user_id' => $this->user->id]);

    $this->actingAs($this->user)
        ->post(route('strategies.deactivate', $strategy))
        ->assertRedirect();

    expect($strategy->fresh()->is_active)->toBeFalse();
});

it('requires authentication', function () {
    $this->get(route('strategies.index'))->assertRedirect('/login');
});
```

**Step 5: Run tests**

```bash
cd web && php artisan test --compact --filter=StrategyControllerTest
```

**Step 6: Commit**

```bash
cd web && vendor/bin/pint --dirty --format agent
git add app/Http/Controllers/StrategyController.php routes/web.php tests/Feature/StrategyControllerTest.php
git commit -m "feat(controllers): add StrategyController with CRUD, activate/deactivate, and tests"
```

---

## Task 9: WalletController + Routes

**Files:**
- Create: `web/app/Http/Controllers/WalletController.php`
- Modify: `web/routes/web.php`
- Create: `web/tests/Feature/WalletControllerTest.php`

**Step 1: Create controller**

```bash
cd web && php artisan make:controller WalletController --no-interaction
```

**Step 2: Implement WalletController**

```php
<?php

namespace App\Http\Controllers;

use App\Http\Requests\AssignStrategyRequest;
use App\Http\Requests\StoreWalletRequest;
use App\Models\Strategy;
use App\Models\Wallet;
use App\Services\WalletService;
use Illuminate\Http\RedirectResponse;
use Inertia\Inertia;
use Inertia\Response;

class WalletController extends Controller
{
    public function index(): Response
    {
        return Inertia::render('wallets/index', [
            'wallets' => auth()->user()->wallets()
                ->withCount('strategies')
                ->get(),
        ]);
    }

    public function store(StoreWalletRequest $request, WalletService $walletService): RedirectResponse
    {
        $keypair = $walletService->generateKeypair();

        $request->user()->wallets()->create([
            'label' => $request->validated('label'),
            'address' => $keypair['address'],
            'private_key_enc' => $keypair['private_key_enc'],
        ]);

        return back()->with('success', 'Wallet created.');
    }

    public function destroy(Wallet $wallet): RedirectResponse
    {
        $this->authorize('delete', $wallet);

        $wallet->delete();

        return to_route('wallets.index')->with('success', 'Wallet deleted.');
    }

    public function assignStrategy(AssignStrategyRequest $request, Wallet $wallet): RedirectResponse
    {
        $this->authorize('view', $wallet);

        $strategy = Strategy::findOrFail($request->validated('strategy_id'));
        $this->authorize('view', $strategy);

        $wallet->strategies()->syncWithoutDetaching([
            $strategy->id => [
                'markets' => $request->validated('markets', []),
                'max_position_usdc' => $request->validated('max_position_usdc', 100),
            ],
        ]);

        return back()->with('success', 'Strategy assigned to wallet.');
    }

    public function removeStrategy(Wallet $wallet, Strategy $strategy): RedirectResponse
    {
        $this->authorize('view', $wallet);

        $wallet->strategies()->detach($strategy->id);

        return back()->with('success', 'Strategy removed from wallet.');
    }
}
```

**Step 3: Add routes**

Add inside the auth middleware group in `routes/web.php`:

```php
use App\Http\Controllers\WalletController;

// Wallets
Route::get('wallets', [WalletController::class, 'index'])->name('wallets.index');
Route::post('wallets', [WalletController::class, 'store'])->name('wallets.store');
Route::delete('wallets/{wallet}', [WalletController::class, 'destroy'])->name('wallets.destroy');
Route::post('wallets/{wallet}/strategies', [WalletController::class, 'assignStrategy'])->name('wallets.assign-strategy');
Route::delete('wallets/{wallet}/strategies/{strategy}', [WalletController::class, 'removeStrategy'])->name('wallets.remove-strategy');
```

**Step 4: Write feature tests**

```bash
cd web && php artisan make:test --pest WalletControllerTest --no-interaction
```

```php
<?php

use App\Models\Strategy;
use App\Models\User;
use App\Models\Wallet;
use App\Services\WalletService;
use Inertia\Testing\AssertableInertia as Assert;

beforeEach(function () {
    $this->user = User::factory()->create();
});

it('displays wallets index page', function () {
    Wallet::factory()->count(2)->create(['user_id' => $this->user->id]);

    $this->actingAs($this->user)
        ->get(route('wallets.index'))
        ->assertOk()
        ->assertInertia(fn (Assert $page) => $page
            ->component('wallets/index')
            ->has('wallets', 2)
        );
});

it('creates a new wallet with generated keypair', function () {
    $mock = Mockery::mock(WalletService::class);
    $mock->shouldReceive('generateKeypair')->once()->andReturn([
        'address' => '0xAbCdEf1234567890AbCdEf1234567890AbCdEf12',
        'private_key_enc' => base64_encode('encrypted-key'),
    ]);
    $this->app->instance(WalletService::class, $mock);

    $this->actingAs($this->user)
        ->post(route('wallets.store'), ['label' => 'My Wallet'])
        ->assertRedirect();

    expect(Wallet::where('user_id', $this->user->id)->count())->toBe(1)
        ->and(Wallet::first()->label)->toBe('My Wallet')
        ->and(Wallet::first()->address)->toBe('0xAbCdEf1234567890AbCdEf1234567890AbCdEf12');
});

it('deletes a wallet belonging to the user', function () {
    $wallet = Wallet::factory()->create(['user_id' => $this->user->id]);

    $this->actingAs($this->user)
        ->delete(route('wallets.destroy', $wallet))
        ->assertRedirect(route('wallets.index'));

    expect(Wallet::find($wallet->id))->toBeNull();
});

it('prevents deleting another users wallet', function () {
    $other = User::factory()->create();
    $wallet = Wallet::factory()->create(['user_id' => $other->id]);

    $this->actingAs($this->user)
        ->delete(route('wallets.destroy', $wallet))
        ->assertForbidden();
});

it('assigns a strategy to a wallet', function () {
    $wallet = Wallet::factory()->create(['user_id' => $this->user->id]);
    $strategy = Strategy::factory()->create(['user_id' => $this->user->id]);

    $this->actingAs($this->user)
        ->post(route('wallets.assign-strategy', $wallet), [
            'strategy_id' => $strategy->id,
            'markets' => ['btc-15m'],
            'max_position_usdc' => 200,
        ])
        ->assertRedirect();

    expect($wallet->strategies()->count())->toBe(1);
});

it('removes a strategy from a wallet', function () {
    $wallet = Wallet::factory()->create(['user_id' => $this->user->id]);
    $strategy = Strategy::factory()->create(['user_id' => $this->user->id]);
    $wallet->strategies()->attach($strategy->id, ['markets' => [], 'max_position_usdc' => 100]);

    $this->actingAs($this->user)
        ->delete(route('wallets.remove-strategy', [$wallet, $strategy]))
        ->assertRedirect();

    expect($wallet->strategies()->count())->toBe(0);
});
```

**Step 5: Run tests**

```bash
cd web && php artisan test --compact --filter=WalletControllerTest
```

**Step 6: Commit**

```bash
cd web && vendor/bin/pint --dirty --format agent
git add app/Http/Controllers/WalletController.php routes/web.php tests/Feature/WalletControllerTest.php
git commit -m "feat(controllers): add WalletController with CRUD, strategy assignment, and tests"
```

---

## Task 10: BacktestController + Routes

**Files:**
- Create: `web/app/Http/Controllers/BacktestController.php`
- Modify: `web/routes/web.php`
- Create: `web/tests/Feature/BacktestControllerTest.php`

**Step 1: Create controller**

```bash
cd web && php artisan make:controller BacktestController --no-interaction
```

**Step 2: Implement BacktestController**

```php
<?php

namespace App\Http\Controllers;

use App\Http\Requests\RunBacktestRequest;
use App\Models\BacktestResult;
use App\Models\Strategy;
use App\Services\EngineService;
use Illuminate\Http\RedirectResponse;
use Inertia\Inertia;
use Inertia\Response;

class BacktestController extends Controller
{
    public function index(): Response
    {
        return Inertia::render('backtests/index', [
            'results' => auth()->user()->backtestResults()
                ->with('strategy:id,name')
                ->latest()
                ->get(),
        ]);
    }

    public function show(BacktestResult $result): Response
    {
        $this->authorize('view', $result);

        $result->load('strategy:id,name,graph');

        return Inertia::render('backtests/show', [
            'result' => $result,
        ]);
    }

    public function run(RunBacktestRequest $request, Strategy $strategy, EngineService $engine): RedirectResponse
    {
        $validated = $request->validated();

        $engineResult = $engine->runBacktest(
            $strategy->graph,
            $validated['market_filter'] ?? [],
            $validated['date_from'],
            $validated['date_to'],
        );

        $result = BacktestResult::create([
            'user_id' => $request->user()->id,
            'strategy_id' => $strategy->id,
            'market_filter' => $validated['market_filter'] ?? null,
            'date_from' => $validated['date_from'],
            'date_to' => $validated['date_to'],
            'total_trades' => $engineResult['total_trades'] ?? null,
            'win_rate' => $engineResult['win_rate'] ?? null,
            'total_pnl_usdc' => $engineResult['pnl'] ?? null,
            'max_drawdown' => $engineResult['max_drawdown'] ?? null,
            'sharpe_ratio' => $engineResult['sharpe_ratio'] ?? null,
            'result_detail' => $engineResult,
        ]);

        return to_route('backtests.show', $result)->with('success', 'Backtest completed.');
    }
}
```

**Step 3: Add routes**

Add inside the auth middleware group in `routes/web.php`:

```php
use App\Http\Controllers\BacktestController;

// Backtests
Route::get('backtests', [BacktestController::class, 'index'])->name('backtests.index');
Route::get('backtests/{result}', [BacktestController::class, 'show'])->name('backtests.show');
Route::post('strategies/{strategy}/backtest', [BacktestController::class, 'run'])->name('backtests.run');
```

**Step 4: Write feature tests**

```bash
cd web && php artisan make:test --pest BacktestControllerTest --no-interaction
```

```php
<?php

use App\Models\BacktestResult;
use App\Models\Strategy;
use App\Models\User;
use Illuminate\Support\Facades\Http;
use Inertia\Testing\AssertableInertia as Assert;

beforeEach(function () {
    $this->user = User::factory()->create();
});

it('displays backtests index page', function () {
    $strategy = Strategy::factory()->create(['user_id' => $this->user->id]);
    BacktestResult::factory()->count(2)->create([
        'user_id' => $this->user->id,
        'strategy_id' => $strategy->id,
    ]);

    $this->actingAs($this->user)
        ->get(route('backtests.index'))
        ->assertOk()
        ->assertInertia(fn (Assert $page) => $page
            ->component('backtests/index')
            ->has('results', 2)
        );
});

it('shows a backtest result', function () {
    $strategy = Strategy::factory()->create(['user_id' => $this->user->id]);
    $result = BacktestResult::factory()->create([
        'user_id' => $this->user->id,
        'strategy_id' => $strategy->id,
    ]);

    $this->actingAs($this->user)
        ->get(route('backtests.show', $result))
        ->assertOk()
        ->assertInertia(fn (Assert $page) => $page
            ->component('backtests/show')
            ->has('result')
        );
});

it('prevents viewing another users backtest result', function () {
    $other = User::factory()->create();
    $strategy = Strategy::factory()->create(['user_id' => $other->id]);
    $result = BacktestResult::factory()->create([
        'user_id' => $other->id,
        'strategy_id' => $strategy->id,
    ]);

    $this->actingAs($this->user)
        ->get(route('backtests.show', $result))
        ->assertForbidden();
});

it('runs a backtest via the engine and stores the result', function () {
    Http::fake(['*/internal/backtest/run' => Http::response([
        'total_trades' => 42,
        'win_rate' => 0.65,
        'pnl' => 123.45,
        'max_drawdown' => 0.12,
        'sharpe_ratio' => 1.5,
        'trades' => [],
    ])]);

    $strategy = Strategy::factory()->create(['user_id' => $this->user->id]);

    $this->actingAs($this->user)
        ->post(route('backtests.run', $strategy), [
            'date_from' => '2026-01-01',
            'date_to' => '2026-02-01',
        ])
        ->assertRedirect();

    $result = BacktestResult::where('strategy_id', $strategy->id)->first();
    expect($result)->not->toBeNull()
        ->and($result->total_trades)->toBe(42)
        ->and($result->win_rate)->toBe('0.6500');
});

it('validates backtest request fields', function () {
    $strategy = Strategy::factory()->create(['user_id' => $this->user->id]);

    $this->actingAs($this->user)
        ->post(route('backtests.run', $strategy), [])
        ->assertSessionHasErrors(['date_from', 'date_to']);
});
```

**Step 5: Run tests**

```bash
cd web && php artisan test --compact --filter=BacktestControllerTest
```

**Step 6: Commit**

```bash
cd web && vendor/bin/pint --dirty --format agent
git add app/Http/Controllers/BacktestController.php routes/web.php tests/Feature/BacktestControllerTest.php
git commit -m "feat(controllers): add BacktestController with run, index, show, and tests"
```

---

## Task 11: CheckPlanLimits Middleware

**Files:**
- Create: `web/app/Http/Middleware/CheckPlanLimits.php`
- Modify: `web/bootstrap/app.php`
- Modify: `web/routes/web.php`
- Create: `web/tests/Feature/CheckPlanLimitsTest.php`

**Step 1: Create middleware**

```bash
cd web && php artisan make:middleware CheckPlanLimits --no-interaction
```

**Step 2: Implement CheckPlanLimits**

```php
<?php

namespace App\Http\Middleware;

use Closure;
use Illuminate\Http\Request;
use Symfony\Component\HttpFoundation\Response;

class CheckPlanLimits
{
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
                $user->wallets()
                    ->withCount('copyRelationships')
                    ->get()
                    ->sum('copy_relationships_count'),
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
```

**Step 3: Register middleware alias in bootstrap/app.php**

Add to the `withMiddleware` closure:

```php
$middleware->alias([
    'plan.limit' => \App\Http\Middleware\CheckPlanLimits::class,
]);
```

**Step 4: Apply to routes in web.php**

Update the wallet store and strategy store routes:

```php
Route::post('wallets', [WalletController::class, 'store'])->name('wallets.store')->middleware('plan.limit:wallets');
Route::post('strategies', [StrategyController::class, 'store'])->name('strategies.store')->middleware('plan.limit:strategies');
```

Note: If using `Route::resource()` for strategies, extract the `store` route separately to apply middleware, or use the controller's `__construct` method. The simplest approach is to check the limit in the controller itself rather than middleware for `Route::resource`. Alternative: keep the middleware on explicit routes and remove from resource.

**Step 5: Write feature tests**

```bash
cd web && php artisan make:test --pest CheckPlanLimitsTest --no-interaction
```

```php
<?php

use App\Models\User;
use App\Models\Wallet;
use App\Models\Strategy;
use App\Services\WalletService;

beforeEach(function () {
    $this->user = User::factory()->create(['plan' => 'free']);

    $mock = Mockery::mock(WalletService::class);
    $mock->shouldReceive('generateKeypair')->andReturn([
        'address' => '0x' . fake()->regexify('[a-fA-F0-9]{40}'),
        'private_key_enc' => base64_encode('encrypted'),
    ]);
    $this->app->instance(WalletService::class, $mock);
});

it('allows creating a wallet when under limit', function () {
    $this->actingAs($this->user)
        ->post(route('wallets.store'), ['label' => 'Wallet 1'])
        ->assertRedirect();

    expect(Wallet::where('user_id', $this->user->id)->count())->toBe(1);
});

it('blocks creating a wallet when at limit', function () {
    Wallet::factory()->create(['user_id' => $this->user->id]);

    $this->actingAs($this->user)
        ->post(route('wallets.store'), ['label' => 'Wallet 2'])
        ->assertRedirect()
        ->assertSessionHas('error');

    expect(Wallet::where('user_id', $this->user->id)->count())->toBe(1);
});

it('allows pro plan users to create many wallets', function () {
    $this->user->update(['plan' => 'pro']);
    Wallet::factory()->count(10)->create(['user_id' => $this->user->id]);

    $this->actingAs($this->user)
        ->post(route('wallets.store'), ['label' => 'Wallet 11'])
        ->assertRedirect()
        ->assertSessionMissing('error');
});

it('blocks creating strategies when at limit for free plan', function () {
    Strategy::factory()->count(2)->create(['user_id' => $this->user->id]);

    $this->actingAs($this->user)
        ->post(route('strategies.store'), [
            'name' => 'Blocked',
            'mode' => 'form',
            'graph' => ['mode' => 'form', 'conditions' => [], 'action' => [], 'risk' => []],
        ])
        ->assertRedirect()
        ->assertSessionHas('error');
});
```

**Step 6: Run tests**

```bash
cd web && php artisan test --compact --filter=CheckPlanLimitsTest
```

**Step 7: Commit**

```bash
cd web && vendor/bin/pint --dirty --format agent
git add app/Http/Middleware/CheckPlanLimits.php bootstrap/app.php routes/web.php tests/Feature/CheckPlanLimitsTest.php
git commit -m "feat(middleware): add CheckPlanLimits middleware enforcing subscription tier limits"
```

---

## Task 12: Stripe Cashier — Billing Setup + BillingController

**Files:**
- Modify: `web/composer.json` (install Cashier)
- Modify: `web/app/Models/User.php`
- Create: `web/app/Http/Controllers/BillingController.php`
- Modify: `web/routes/web.php`
- Create: `web/tests/Feature/BillingControllerTest.php`

**Step 1: Install Laravel Cashier**

```bash
cd web && composer require laravel/cashier --no-interaction
```

Do NOT publish Cashier migrations — the subscription tables already exist from Phase 1 migrations.

**Step 2: Add Billable trait to User model**

Add to User model imports and trait usage:

```php
use Laravel\Cashier\Billable;

class User extends Authenticatable
{
    use Billable, HasFactory, Notifiable, TwoFactorAuthenticatable;
    // ...
}
```

**Step 3: Add Cashier config to .env.example**

Ensure these exist (they should from Phase 1):
```
STRIPE_KEY=
STRIPE_SECRET=
STRIPE_WEBHOOK_SECRET=
```

**Step 4: Create BillingController**

```bash
cd web && php artisan make:controller BillingController --no-interaction
```

```php
<?php

namespace App\Http\Controllers;

use Illuminate\Http\RedirectResponse;
use Illuminate\Http\Request;
use Inertia\Inertia;
use Inertia\Response;

class BillingController extends Controller
{
    public function index(Request $request): Response
    {
        $user = $request->user();

        return Inertia::render('billing/index', [
            'plan' => $user->plan ?? 'free',
            'subscription' => $user->subscription('default'),
            'onTrial' => $user->onTrial('default'),
            'subscribed' => $user->subscribed('default'),
        ]);
    }

    public function subscribe(Request $request): RedirectResponse
    {
        $validated = $request->validate([
            'price_id' => ['required', 'string'],
        ]);

        return $request->user()
            ->newSubscription('default', $validated['price_id'])
            ->checkout([
                'success_url' => route('billing.index') . '?checkout=success',
                'cancel_url' => route('billing.index') . '?checkout=cancelled',
            ])
            ->redirect();
    }

    public function portal(Request $request): RedirectResponse
    {
        return $request->user()->redirectToBillingPortal(route('billing.index'));
    }
}
```

**Step 5: Add billing routes**

Add inside the auth middleware group in `routes/web.php`:

```php
use App\Http\Controllers\BillingController;

// Billing
Route::get('billing', [BillingController::class, 'index'])->name('billing.index');
Route::post('billing/subscribe', [BillingController::class, 'subscribe'])->name('billing.subscribe');
Route::post('billing/portal', [BillingController::class, 'portal'])->name('billing.portal');
```

Add the Stripe webhook route outside auth middleware:

```php
// Stripe Webhook (no auth)
Route::stripeWebhooks('webhooks/stripe');
```

Note: `Route::stripeWebhooks()` requires `spatie/laravel-stripe-webhooks` or you can use Cashier's built-in webhook handling. For Cashier, add this to `bootstrap/app.php` or use Cashier's webhook controller directly. The simplest approach for Cashier v15:

```php
// In routes/web.php, outside auth group:
Route::post('webhooks/stripe', [\Laravel\Cashier\Http\Controllers\WebhookController::class, 'handleWebhook'])
    ->name('cashier.webhook');
```

**Step 6: Write feature tests**

```bash
cd web && php artisan make:test --pest BillingControllerTest --no-interaction
```

```php
<?php

use App\Models\User;
use Inertia\Testing\AssertableInertia as Assert;

beforeEach(function () {
    $this->user = User::factory()->create();
});

it('displays billing page', function () {
    $this->actingAs($this->user)
        ->get(route('billing.index'))
        ->assertOk()
        ->assertInertia(fn (Assert $page) => $page
            ->component('billing/index')
            ->has('plan')
            ->has('subscribed')
        );
});

it('validates price_id on subscribe', function () {
    $this->actingAs($this->user)
        ->post(route('billing.subscribe'), [])
        ->assertSessionHasErrors(['price_id']);
});

it('requires authentication for billing pages', function () {
    $this->get(route('billing.index'))->assertRedirect('/login');
});
```

**Step 7: Run tests**

```bash
cd web && php artisan test --compact --filter=BillingControllerTest
```

**Step 8: Commit**

```bash
cd web && vendor/bin/pint --dirty --format agent
git add app/Models/User.php app/Http/Controllers/BillingController.php routes/web.php tests/Feature/BillingControllerTest.php composer.json composer.lock
git commit -m "feat(billing): add Stripe Cashier integration with BillingController"
```

---

## Task 13: Basic Inertia Pages — Strategies

**Files:**
- Create: `web/resources/js/pages/strategies/index.tsx`
- Create: `web/resources/js/pages/strategies/create.tsx`
- Create: `web/resources/js/pages/strategies/show.tsx`

These are minimal functional pages. Phase 8 will add the full strategy builder UI with form mode and node editor.

**Step 1: Create strategies/index.tsx**

```tsx
import { Head, Link, router } from '@inertiajs/react';
import AppLayout from '@/layouts/app-layout';
import { Button } from '@/components/ui/button';
import type { BreadcrumbItem } from '@/types';

const breadcrumbs: BreadcrumbItem[] = [
    { title: 'Strategies', href: '/strategies' },
];

interface Strategy {
    id: number;
    name: string;
    mode: string;
    is_active: boolean;
    wallets_count: number;
    created_at: string;
}

export default function StrategiesIndex({ strategies }: { strategies: Strategy[] }) {
    return (
        <AppLayout breadcrumbs={breadcrumbs}>
            <Head title="Strategies" />
            <div className="p-6">
                <div className="mb-6 flex items-center justify-between">
                    <h1 className="text-2xl font-bold">Strategies</h1>
                    <Link href="/strategies/create">
                        <Button>New Strategy</Button>
                    </Link>
                </div>
                <div className="space-y-3">
                    {strategies.length === 0 && (
                        <p className="text-muted-foreground">No strategies yet. Create your first one.</p>
                    )}
                    {strategies.map((strategy) => (
                        <Link
                            key={strategy.id}
                            href={`/strategies/${strategy.id}`}
                            className="border-sidebar-border block rounded-lg border p-4 transition hover:bg-accent"
                        >
                            <div className="flex items-center justify-between">
                                <div>
                                    <h3 className="font-semibold">{strategy.name}</h3>
                                    <p className="text-muted-foreground text-sm">
                                        {strategy.mode} mode · {strategy.wallets_count} wallet(s)
                                    </p>
                                </div>
                                <span className={`rounded-full px-2 py-1 text-xs font-medium ${strategy.is_active ? 'bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300' : 'bg-gray-100 text-gray-600 dark:bg-gray-800 dark:text-gray-400'}`}>
                                    {strategy.is_active ? 'Active' : 'Inactive'}
                                </span>
                            </div>
                        </Link>
                    ))}
                </div>
            </div>
        </AppLayout>
    );
}
```

**Step 2: Create strategies/create.tsx**

```tsx
import { Head, useForm } from '@inertiajs/react';
import AppLayout from '@/layouts/app-layout';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import type { BreadcrumbItem } from '@/types';

const breadcrumbs: BreadcrumbItem[] = [
    { title: 'Strategies', href: '/strategies' },
    { title: 'Create', href: '/strategies/create' },
];

export default function StrategiesCreate() {
    const { data, setData, post, processing, errors } = useForm({
        name: '',
        description: '',
        mode: 'form',
        graph: {
            mode: 'form',
            conditions: [],
            action: { signal: 'buy', outcome: 'UP', size_mode: 'fixed', size_usdc: 50, order_type: 'market' },
            risk: { stoploss_pct: 30, take_profit_pct: 80, max_position_usdc: 200, max_trades_per_slot: 1 },
        },
    });

    function handleSubmit(e: React.FormEvent) {
        e.preventDefault();
        post('/strategies');
    }

    return (
        <AppLayout breadcrumbs={breadcrumbs}>
            <Head title="Create Strategy" />
            <div className="mx-auto max-w-2xl p-6">
                <h1 className="mb-6 text-2xl font-bold">Create Strategy</h1>
                <form onSubmit={handleSubmit} className="space-y-4">
                    <div>
                        <Label htmlFor="name">Name</Label>
                        <Input
                            id="name"
                            value={data.name}
                            onChange={(e) => setData('name', e.target.value)}
                        />
                        {errors.name && <p className="mt-1 text-sm text-red-500">{errors.name}</p>}
                    </div>
                    <div>
                        <Label htmlFor="description">Description</Label>
                        <Input
                            id="description"
                            value={data.description}
                            onChange={(e) => setData('description', e.target.value)}
                        />
                    </div>
                    <p className="text-muted-foreground text-sm">
                        Strategy builder will be available in a future update. A default strategy configuration is used for now.
                    </p>
                    <Button type="submit" disabled={processing}>
                        Create Strategy
                    </Button>
                </form>
            </div>
        </AppLayout>
    );
}
```

**Step 3: Create strategies/show.tsx**

```tsx
import { Head, router } from '@inertiajs/react';
import AppLayout from '@/layouts/app-layout';
import { Button } from '@/components/ui/button';
import type { BreadcrumbItem } from '@/types';

interface WalletStrategy {
    id: number;
    is_running: boolean;
    max_position_usdc: string;
    wallet: { id: number; label: string | null; address: string };
}

interface BacktestResult {
    id: number;
    total_trades: number;
    win_rate: string;
    total_pnl_usdc: string;
    created_at: string;
}

interface Strategy {
    id: number;
    name: string;
    description: string | null;
    mode: string;
    graph: Record<string, unknown>;
    is_active: boolean;
    wallet_strategies: WalletStrategy[];
    backtest_results: BacktestResult[];
}

export default function StrategiesShow({ strategy }: { strategy: Strategy }) {
    const breadcrumbs: BreadcrumbItem[] = [
        { title: 'Strategies', href: '/strategies' },
        { title: strategy.name, href: `/strategies/${strategy.id}` },
    ];

    return (
        <AppLayout breadcrumbs={breadcrumbs}>
            <Head title={strategy.name} />
            <div className="p-6">
                <div className="mb-6 flex items-center justify-between">
                    <div>
                        <h1 className="text-2xl font-bold">{strategy.name}</h1>
                        {strategy.description && (
                            <p className="text-muted-foreground mt-1">{strategy.description}</p>
                        )}
                    </div>
                    <div className="flex gap-2">
                        {strategy.is_active ? (
                            <Button
                                variant="outline"
                                onClick={() => router.post(`/strategies/${strategy.id}/deactivate`)}
                            >
                                Deactivate
                            </Button>
                        ) : (
                            <Button
                                onClick={() => router.post(`/strategies/${strategy.id}/activate`)}
                            >
                                Activate
                            </Button>
                        )}
                        <Button
                            variant="destructive"
                            onClick={() => router.delete(`/strategies/${strategy.id}`)}
                        >
                            Delete
                        </Button>
                    </div>
                </div>

                <div className="grid gap-6 md:grid-cols-2">
                    <div className="border-sidebar-border rounded-lg border p-4">
                        <h2 className="mb-3 font-semibold">Configuration</h2>
                        <dl className="space-y-2 text-sm">
                            <div className="flex justify-between">
                                <dt className="text-muted-foreground">Mode</dt>
                                <dd>{strategy.mode}</dd>
                            </div>
                            <div className="flex justify-between">
                                <dt className="text-muted-foreground">Status</dt>
                                <dd>{strategy.is_active ? 'Active' : 'Inactive'}</dd>
                            </div>
                        </dl>
                    </div>

                    <div className="border-sidebar-border rounded-lg border p-4">
                        <h2 className="mb-3 font-semibold">Assigned Wallets</h2>
                        {strategy.wallet_strategies.length === 0 ? (
                            <p className="text-muted-foreground text-sm">No wallets assigned.</p>
                        ) : (
                            <ul className="space-y-2 text-sm">
                                {strategy.wallet_strategies.map((ws) => (
                                    <li key={ws.id} className="flex items-center justify-between">
                                        <span className="font-mono text-xs">
                                            {ws.wallet.label || ws.wallet.address.slice(0, 10) + '...'}
                                        </span>
                                        <span className={ws.is_running ? 'text-green-600' : 'text-gray-400'}>
                                            {ws.is_running ? 'Running' : 'Stopped'}
                                        </span>
                                    </li>
                                ))}
                            </ul>
                        )}
                    </div>
                </div>

                <div className="border-sidebar-border mt-6 rounded-lg border p-4">
                    <h2 className="mb-3 font-semibold">Recent Backtests</h2>
                    {strategy.backtest_results.length === 0 ? (
                        <p className="text-muted-foreground text-sm">No backtests yet.</p>
                    ) : (
                        <div className="overflow-x-auto">
                            <table className="w-full text-sm">
                                <thead>
                                    <tr className="text-muted-foreground border-b text-left">
                                        <th className="pb-2">Date</th>
                                        <th className="pb-2">Trades</th>
                                        <th className="pb-2">Win Rate</th>
                                        <th className="pb-2">PnL</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {strategy.backtest_results.map((bt) => (
                                        <tr key={bt.id} className="border-b">
                                            <td className="py-2">{new Date(bt.created_at).toLocaleDateString()}</td>
                                            <td className="py-2">{bt.total_trades}</td>
                                            <td className="py-2">{(parseFloat(bt.win_rate) * 100).toFixed(1)}%</td>
                                            <td className={`py-2 ${parseFloat(bt.total_pnl_usdc) >= 0 ? 'text-green-600' : 'text-red-600'}`}>
                                                ${parseFloat(bt.total_pnl_usdc).toFixed(2)}
                                            </td>
                                        </tr>
                                    ))}
                                </tbody>
                            </table>
                        </div>
                    )}
                </div>
            </div>
        </AppLayout>
    );
}
```

**Step 4: Commit**

```bash
cd web && npx prettier --write resources/js/pages/strategies/
git add resources/js/pages/strategies/
git commit -m "feat(pages): add basic strategy Inertia pages (index, create, show)"
```

---

## Task 14: Basic Inertia Pages — Wallets, Backtests, Billing

**Files:**
- Create: `web/resources/js/pages/wallets/index.tsx`
- Create: `web/resources/js/pages/backtests/index.tsx`
- Create: `web/resources/js/pages/backtests/show.tsx`
- Create: `web/resources/js/pages/billing/index.tsx`

**Step 1: Create wallets/index.tsx**

```tsx
import { Head, router, useForm } from '@inertiajs/react';
import AppLayout from '@/layouts/app-layout';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import type { BreadcrumbItem } from '@/types';

const breadcrumbs: BreadcrumbItem[] = [
    { title: 'Wallets', href: '/wallets' },
];

interface Wallet {
    id: number;
    label: string | null;
    address: string;
    balance_usdc: string;
    is_active: boolean;
    strategies_count: number;
}

export default function WalletsIndex({ wallets }: { wallets: Wallet[] }) {
    const { data, setData, post, processing, reset } = useForm({ label: '' });

    function handleSubmit(e: React.FormEvent) {
        e.preventDefault();
        post('/wallets', { onSuccess: () => reset() });
    }

    return (
        <AppLayout breadcrumbs={breadcrumbs}>
            <Head title="Wallets" />
            <div className="p-6">
                <h1 className="mb-6 text-2xl font-bold">Wallets</h1>

                <form onSubmit={handleSubmit} className="mb-6 flex items-end gap-3">
                    <div>
                        <Label htmlFor="label">Label (optional)</Label>
                        <Input
                            id="label"
                            value={data.label}
                            onChange={(e) => setData('label', e.target.value)}
                            placeholder="My trading wallet"
                        />
                    </div>
                    <Button type="submit" disabled={processing}>Generate Wallet</Button>
                </form>

                <div className="space-y-3">
                    {wallets.length === 0 && (
                        <p className="text-muted-foreground">No wallets yet. Generate your first one above.</p>
                    )}
                    {wallets.map((wallet) => (
                        <div key={wallet.id} className="border-sidebar-border flex items-center justify-between rounded-lg border p-4">
                            <div>
                                <h3 className="font-semibold">{wallet.label || 'Unnamed Wallet'}</h3>
                                <p className="font-mono text-muted-foreground text-xs">{wallet.address}</p>
                                <p className="text-muted-foreground mt-1 text-sm">
                                    ${parseFloat(wallet.balance_usdc).toFixed(2)} USDC · {wallet.strategies_count} strateg{wallet.strategies_count === 1 ? 'y' : 'ies'}
                                </p>
                            </div>
                            <Button
                                variant="destructive"
                                size="sm"
                                onClick={() => router.delete(`/wallets/${wallet.id}`)}
                            >
                                Delete
                            </Button>
                        </div>
                    ))}
                </div>
            </div>
        </AppLayout>
    );
}
```

**Step 2: Create backtests/index.tsx**

```tsx
import { Head, Link } from '@inertiajs/react';
import AppLayout from '@/layouts/app-layout';
import type { BreadcrumbItem } from '@/types';

const breadcrumbs: BreadcrumbItem[] = [
    { title: 'Backtests', href: '/backtests' },
];

interface Result {
    id: number;
    total_trades: number | null;
    win_rate: string | null;
    total_pnl_usdc: string | null;
    created_at: string;
    strategy: { id: number; name: string };
}

export default function BacktestsIndex({ results }: { results: Result[] }) {
    return (
        <AppLayout breadcrumbs={breadcrumbs}>
            <Head title="Backtests" />
            <div className="p-6">
                <h1 className="mb-6 text-2xl font-bold">Backtest Results</h1>
                {results.length === 0 ? (
                    <p className="text-muted-foreground">No backtest results yet. Run one from a strategy page.</p>
                ) : (
                    <div className="overflow-x-auto">
                        <table className="w-full text-sm">
                            <thead>
                                <tr className="text-muted-foreground border-b text-left">
                                    <th className="pb-2">Strategy</th>
                                    <th className="pb-2">Trades</th>
                                    <th className="pb-2">Win Rate</th>
                                    <th className="pb-2">PnL</th>
                                    <th className="pb-2">Date</th>
                                </tr>
                            </thead>
                            <tbody>
                                {results.map((r) => (
                                    <tr key={r.id} className="border-b">
                                        <td className="py-2">
                                            <Link href={`/backtests/${r.id}`} className="text-blue-600 hover:underline">
                                                {r.strategy.name}
                                            </Link>
                                        </td>
                                        <td className="py-2">{r.total_trades ?? '-'}</td>
                                        <td className="py-2">{r.win_rate ? `${(parseFloat(r.win_rate) * 100).toFixed(1)}%` : '-'}</td>
                                        <td className={`py-2 ${r.total_pnl_usdc && parseFloat(r.total_pnl_usdc) >= 0 ? 'text-green-600' : 'text-red-600'}`}>
                                            {r.total_pnl_usdc ? `$${parseFloat(r.total_pnl_usdc).toFixed(2)}` : '-'}
                                        </td>
                                        <td className="py-2">{new Date(r.created_at).toLocaleDateString()}</td>
                                    </tr>
                                ))}
                            </tbody>
                        </table>
                    </div>
                )}
            </div>
        </AppLayout>
    );
}
```

**Step 3: Create backtests/show.tsx**

```tsx
import { Head } from '@inertiajs/react';
import AppLayout from '@/layouts/app-layout';
import type { BreadcrumbItem } from '@/types';

interface BacktestResult {
    id: number;
    total_trades: number | null;
    win_rate: string | null;
    total_pnl_usdc: string | null;
    max_drawdown: string | null;
    sharpe_ratio: string | null;
    date_from: string | null;
    date_to: string | null;
    created_at: string;
    strategy: { id: number; name: string };
}

export default function BacktestsShow({ result }: { result: BacktestResult }) {
    const breadcrumbs: BreadcrumbItem[] = [
        { title: 'Backtests', href: '/backtests' },
        { title: `#${result.id}`, href: `/backtests/${result.id}` },
    ];

    const metrics = [
        { label: 'Total Trades', value: result.total_trades ?? '-' },
        { label: 'Win Rate', value: result.win_rate ? `${(parseFloat(result.win_rate) * 100).toFixed(1)}%` : '-' },
        { label: 'PnL', value: result.total_pnl_usdc ? `$${parseFloat(result.total_pnl_usdc).toFixed(2)}` : '-' },
        { label: 'Max Drawdown', value: result.max_drawdown ? `${(parseFloat(result.max_drawdown) * 100).toFixed(1)}%` : '-' },
        { label: 'Sharpe Ratio', value: result.sharpe_ratio ?? '-' },
    ];

    return (
        <AppLayout breadcrumbs={breadcrumbs}>
            <Head title={`Backtest #${result.id}`} />
            <div className="p-6">
                <h1 className="mb-2 text-2xl font-bold">Backtest #{result.id}</h1>
                <p className="text-muted-foreground mb-6">
                    Strategy: {result.strategy.name}
                    {result.date_from && result.date_to && (
                        <> · {new Date(result.date_from).toLocaleDateString()} – {new Date(result.date_to).toLocaleDateString()}</>
                    )}
                </p>
                <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-5">
                    {metrics.map((m) => (
                        <div key={m.label} className="border-sidebar-border rounded-lg border p-4">
                            <p className="text-muted-foreground text-sm">{m.label}</p>
                            <p className="text-2xl font-bold">{m.value}</p>
                        </div>
                    ))}
                </div>
            </div>
        </AppLayout>
    );
}
```

**Step 4: Create billing/index.tsx**

```tsx
import { Head, router } from '@inertiajs/react';
import AppLayout from '@/layouts/app-layout';
import { Button } from '@/components/ui/button';
import type { BreadcrumbItem } from '@/types';

const breadcrumbs: BreadcrumbItem[] = [
    { title: 'Billing', href: '/billing' },
];

const plans = [
    { key: 'free', name: 'Free', price: '$0/mo', wallets: '1', strategies: '2' },
    { key: 'starter', name: 'Starter', price: '$29/mo', wallets: '5', strategies: '10' },
    { key: 'pro', name: 'Pro', price: '$79/mo', wallets: '25', strategies: 'Unlimited' },
    { key: 'enterprise', name: 'Enterprise', price: '$249/mo', wallets: 'Unlimited', strategies: 'Unlimited' },
];

interface Props {
    plan: string;
    subscribed: boolean;
}

export default function BillingIndex({ plan, subscribed }: Props) {
    return (
        <AppLayout breadcrumbs={breadcrumbs}>
            <Head title="Billing" />
            <div className="p-6">
                <h1 className="mb-6 text-2xl font-bold">Billing</h1>

                <div className="mb-6 grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
                    {plans.map((p) => (
                        <div
                            key={p.key}
                            className={`border-sidebar-border rounded-lg border p-4 ${plan === p.key ? 'ring-primary ring-2' : ''}`}
                        >
                            <h3 className="font-semibold">{p.name}</h3>
                            <p className="mt-1 text-2xl font-bold">{p.price}</p>
                            <ul className="text-muted-foreground mt-3 space-y-1 text-sm">
                                <li>{p.wallets} wallet(s)</li>
                                <li>{p.strategies} strategies</li>
                            </ul>
                            {plan === p.key && (
                                <p className="text-primary mt-3 text-sm font-medium">Current plan</p>
                            )}
                        </div>
                    ))}
                </div>

                {subscribed && (
                    <Button
                        variant="outline"
                        onClick={() => router.post('/billing/portal')}
                    >
                        Manage Subscription
                    </Button>
                )}
            </div>
        </AppLayout>
    );
}
```

**Step 5: Commit**

```bash
cd web && npx prettier --write resources/js/pages/wallets/ resources/js/pages/backtests/ resources/js/pages/billing/
git add resources/js/pages/wallets/ resources/js/pages/backtests/ resources/js/pages/billing/
git commit -m "feat(pages): add basic Inertia pages for wallets, backtests, and billing"
```

---

## Task 15: Final Wiring — Navigation + Route Consolidation

**Files:**
- Modify: `web/routes/web.php` (final consolidated version)
- Verify all tests pass
- Run Pint

**Step 1: Finalize routes/web.php**

The final `routes/web.php` should look like this:

```php
<?php

use App\Http\Controllers\BacktestController;
use App\Http\Controllers\BillingController;
use App\Http\Controllers\StrategyController;
use App\Http\Controllers\WalletController;
use Illuminate\Support\Facades\Route;
use Inertia\Inertia;
use Laravel\Fortify\Features;

Route::get('/', function () {
    return Inertia::render('welcome', [
        'canRegister' => Features::enabled(Features::registration()),
    ]);
})->name('home');

Route::middleware(['auth', 'verified'])->group(function () {
    Route::get('dashboard', function () {
        return Inertia::render('dashboard');
    })->name('dashboard');

    // Strategies
    Route::resource('strategies', StrategyController::class)->except(['edit']);
    Route::post('strategies/{strategy}/activate', [StrategyController::class, 'activate'])->name('strategies.activate');
    Route::post('strategies/{strategy}/deactivate', [StrategyController::class, 'deactivate'])->name('strategies.deactivate');

    // Wallets
    Route::get('wallets', [WalletController::class, 'index'])->name('wallets.index');
    Route::post('wallets', [WalletController::class, 'store'])->name('wallets.store')->middleware('plan.limit:wallets');
    Route::delete('wallets/{wallet}', [WalletController::class, 'destroy'])->name('wallets.destroy');
    Route::post('wallets/{wallet}/strategies', [WalletController::class, 'assignStrategy'])->name('wallets.assign-strategy');
    Route::delete('wallets/{wallet}/strategies/{strategy}', [WalletController::class, 'removeStrategy'])->name('wallets.remove-strategy');

    // Backtests
    Route::get('backtests', [BacktestController::class, 'index'])->name('backtests.index');
    Route::get('backtests/{result}', [BacktestController::class, 'show'])->name('backtests.show');
    Route::post('strategies/{strategy}/backtest', [BacktestController::class, 'run'])->name('backtests.run');

    // Billing
    Route::get('billing', [BillingController::class, 'index'])->name('billing.index');
    Route::post('billing/subscribe', [BillingController::class, 'subscribe'])->name('billing.subscribe');
    Route::post('billing/portal', [BillingController::class, 'portal'])->name('billing.portal');
});

// Strategy creation with plan limit
Route::middleware(['auth', 'verified', 'plan.limit:strategies'])->group(function () {
    Route::post('strategies', [StrategyController::class, 'store'])->name('strategies.store');
});

// Stripe Webhook (no auth)
Route::post('webhooks/stripe', [\Laravel\Cashier\Http\Controllers\WebhookController::class, 'handleWebhook'])
    ->name('cashier.webhook');

require __DIR__.'/settings.php';
```

Note: The strategy store route needs to be extracted from the resource to apply plan limit middleware. Adjust the `Route::resource` to `->except(['edit', 'store'])` and add the store route separately with the middleware.

**Step 2: Run all tests**

```bash
cd web && php artisan test --compact
```

All tests should pass. Fix any failures before committing.

**Step 3: Run Pint**

```bash
cd web && vendor/bin/pint --dirty --format agent
```

**Step 4: Final commit**

```bash
git add -A
git commit -m "feat(phase7): finalize routes and navigation wiring"
```

---

## Summary

Phase 7 delivers:
- **8 Eloquent models** with relationships, casts, and factories (Strategy, Wallet, WalletStrategy, Trade, WatchedWallet, CopyRelationship, CopyTrade, BacktestResult)
- **User model** extended with relationships, plan limits, and Billable trait
- **WalletService** with Ethereum keypair generation + AES-256-GCM encryption
- **3 authorization policies** (Strategy, Wallet, BacktestResult)
- **5 form requests** with validation
- **4 controllers** (Strategy, Wallet, Backtest, Billing) with full CRUD
- **CheckPlanLimits middleware** enforcing subscription tier limits
- **Stripe Cashier** integration with webhook handling
- **8 basic Inertia pages** (strategies index/create/show, wallets index, backtests index/show, billing index)
- **Comprehensive test coverage** (unit + feature tests)

Phase 8 will enhance the frontend with the strategy builder UI (form mode + node editor), dashboard with live data, charts, and polished layouts.
