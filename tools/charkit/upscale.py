"""Scale2x (AdvMAME2x / EPX) on index maps: rounds diagonals, invents no colours."""
import numpy as np


def scale2x(a):
    P = np.pad(a, ((1, 1), (1, 1)) + ((0, 0),) * (a.ndim - 2), mode='edge')
    C = P[1:-1, 1:-1]
    A, B, D, E = P[:-2, 1:-1], P[1:-1, 2:], P[1:-1, :-2], P[2:, 1:-1]  # up, right, left, down
    eq = (lambda x, y: (x == y).all(-1)) if a.ndim == 3 else (lambda x, y: x == y)
    out = np.repeat(np.repeat(a, 2, 0), 2, 1)
    nAE, nBD = eq(A, E), eq(B, D)
    keep = ~nAE & ~nBD
    for (r, c), (X, Y) in {(0, 0): (D, A), (0, 1): (A, B), (1, 0): (E, D), (1, 1): (B, E)}.items():
        m = eq(X, Y) & keep
        view = out[r::2, c::2]
        view[m] = X[m]
    return out
