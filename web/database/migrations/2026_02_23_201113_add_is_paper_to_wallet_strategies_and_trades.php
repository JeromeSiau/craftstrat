<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        Schema::table('wallet_strategies', function (Blueprint $table) {
            $table->boolean('is_paper')->default(false)->after('is_running');
        });

        Schema::table('trades', function (Blueprint $table) {
            $table->boolean('is_paper')->default(false)->after('status');
            $table->index(['strategy_id', 'is_paper']);
        });
    }

    public function down(): void
    {
        Schema::table('trades', function (Blueprint $table) {
            $table->dropIndex(['strategy_id', 'is_paper']);
            $table->dropColumn('is_paper');
        });

        Schema::table('wallet_strategies', function (Blueprint $table) {
            $table->dropColumn('is_paper');
        });
    }
};
