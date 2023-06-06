import numpy as np


def transpose(array: np.ndarray) -> np.ndarray:
    if array.ndim == 2:
        return array.transpose()
    elif array.ndim == 3:
        return array.transpose(1, 0, 2)
    else:
        raise ValueError("Wrong array dimension")
