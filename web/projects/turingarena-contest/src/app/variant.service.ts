import { Inject, Injectable, LOCALE_ID } from '@angular/core';

import { TextFragment } from './__generated__/TextFragment';
import { VariantAttributeFragment } from './__generated__/VariantAttributeFragment';

@Injectable({
  providedIn: 'root',
})
export class VariantService {
  constructor(
    @Inject(LOCALE_ID) readonly locale: string,
  ) { }

  selectVariant<T extends { attributes: VariantAttributeFragment[] }>(variants: T[], style?: string): T | undefined {
    const styleScore = 10;
    const languageScore = 5;
    const sortedVariants = variants.slice();

    sortedVariants.sort((a, b) => {
      const [aScore, bScore] = [a, b].map((v) =>
        (v.attributes.some(({ key, value }) => key === 'style' && value === style) ? styleScore : 0)
        +
        (v.attributes.some(({ key, value }) => key === 'locale' && value === this.locale) ? languageScore : 0),
      );

      return bScore - aScore;
    });

    return sortedVariants[0];
  }


  selectTextVariant(variants: TextFragment[], style?: string): string | undefined {
    const variant = this.selectVariant(variants, style);

    return variant !== undefined ? variant.value : '';
  }
}
