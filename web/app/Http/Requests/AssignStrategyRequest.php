<?php

namespace App\Http\Requests;

use Illuminate\Contracts\Validation\ValidationRule;
use Illuminate\Foundation\Http\FormRequest;
use Illuminate\Validation\Rule;

class AssignStrategyRequest extends FormRequest
{
    public function authorize(): bool
    {
        return true;
    }

    /**
     * @return array<string, ValidationRule|array<mixed>|string>
     */
    public function rules(): array
    {
        return [
            'strategy_id' => [
                'required',
                Rule::exists('strategies', 'id')->where('user_id', $this->user()->id),
            ],
            'markets' => ['nullable', 'array'],
            'markets.*' => ['string'],
            'max_position_usdc' => ['nullable', 'numeric', 'min:1', 'max:1000000'],
            'is_paper' => ['nullable', 'boolean'],
        ];
    }
}
