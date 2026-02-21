<?php

namespace App\Http\Requests;

use Illuminate\Contracts\Validation\ValidationRule;
use Illuminate\Foundation\Http\FormRequest;

class RunBacktestRequest extends FormRequest
{
    /**
     * Determine if the user is authorized to make this request.
     */
    public function authorize(): bool
    {
        return $this->user()->can('view', $this->route('strategy'));
    }

    /**
     * Get the validation rules that apply to the request.
     *
     * @return array<string, ValidationRule|array<mixed>|string>
     */
    public function rules(): array
    {
        $limits = $this->user()->planLimits();
        $backtestDays = $limits['backtest_days'];

        $dateFromRules = ['required', 'date'];

        if ($backtestDays !== null) {
            $dateFromRules[] = 'after_or_equal:'.now()->subDays($backtestDays)->toDateString();
        }

        return [
            'market_filter' => ['nullable', 'array'],
            'market_filter.*' => ['string'],
            'date_from' => $dateFromRules,
            'date_to' => ['required', 'date', 'after:date_from'],
        ];
    }

    /**
     * @return array<string, string>
     */
    public function messages(): array
    {
        $limits = $this->user()->planLimits();
        $backtestDays = $limits['backtest_days'];

        if ($backtestDays === null) {
            return [];
        }

        return [
            'date_from.after_or_equal' => "Your plan limits backtests to the last {$backtestDays} days.",
        ];
    }
}
