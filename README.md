# pandas_primes

Sample pandas extension to show how to implement a pandas extension with
Apache Arrow and Rust.

This project is not intended for production use, since it is very simplified
to reduce noise and let readers focus on the concepts presented here. In
particular, error control is mostly lacking both in the Rust and Python
code.

## Example

```python
>>> import pandas
>>> import pandas_prime

>>> data = pandas.Series([1, 2, 3, 4, 5, 6, 7, 8, 9], dtype='uint64[pyarrow]')

>>> data.primes.is_prime()
0    False
1     True
2     True
3    False
4     True
5    False
6     True
7    False
8    False
dtype: bool

>>> data[data.primes.is_prime()]
1   2
2   3
4   5
6   7
dtype: uint64[pyarrow]

>>> data.primes.are_all_primes()
False

>>> data[data.primes.is_prime()].primes.are_all_primes()
True
```
