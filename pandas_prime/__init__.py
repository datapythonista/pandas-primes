import pandas

import arrow_prime


@pandas.api.extensions.register_series_accessor('primes')
class PrimeExtension:
    def __init__(self, input_series):
        self._input_series = input_series

    def is_prime(self):
        input_arrow_array = self._input_series.array._data.chunks[0]
        output_arrow_array = arrow_prime.is_prime(input_arrow_array)
        output_series = pandas.Series(output_arrow_array,
                                      index=self._input_series.index)
        return output_series

    def are_all_primes(self):
        input_arrow_array = self._input_series.array._data.chunks[0]
        output = arrow_prime.are_all_primes(input_arrow_array)
        return output
