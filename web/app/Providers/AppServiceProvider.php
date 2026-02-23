<?php

namespace App\Providers;

use Carbon\CarbonImmutable;
use Illuminate\Cache\RateLimiting\Limit;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Date;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\RateLimiter;
use Illuminate\Support\ServiceProvider;
use Illuminate\Validation\Rules\Password;
use Laravel\Cashier\Cashier;

class AppServiceProvider extends ServiceProvider
{
    /**
     * Register any application services.
     */
    public function register(): void
    {
        $this->app->singleton(\App\Services\EngineService::class, function ($app) {
            return new \App\Services\EngineService(
                baseUrl: config('services.engine.url'),
                timeout: (int) config('services.engine.timeout'),
            );
        });

        $this->app->singleton(\App\Services\WalletService::class, function ($app) {
            return new \App\Services\WalletService(
                encryptionKey: config('services.wallet.encryption_key'),
            );
        });

        $this->app->singleton(\App\Services\StrategyGeneratorService::class, function ($app) {
            return new \App\Services\StrategyGeneratorService(
                apiKey: (string) config('services.anthropic.api_key'),
                model: (string) config('services.anthropic.model'),
            );
        });
    }

    /**
     * Bootstrap any application services.
     */
    public function boot(): void
    {
        Cashier::ignoreRoutes();

        RateLimiter::for('ai-generation', function (Request $request) {
            /** @var \App\Models\User $user */
            $user = $request->user();
            $max = $user->planLimits()['ai_generations_per_day'] ?? 5;

            return $max === null
                ? Limit::none()
                : Limit::perDay($max)->by($user->id);
        });

        $this->configureDefaults();
    }

    /**
     * Configure default behaviors for production-ready applications.
     */
    protected function configureDefaults(): void
    {
        Date::use(CarbonImmutable::class);

        DB::prohibitDestructiveCommands(
            app()->isProduction(),
        );

        Password::defaults(fn (): ?Password => app()->isProduction()
            ? Password::min(12)
                ->mixedCase()
                ->letters()
                ->numbers()
                ->symbols()
                ->uncompromised()
            : null
        );
    }
}
