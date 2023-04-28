import itertools

def align_up(x, m):
    return ((x - 1) | (m - 1)) + 1

def aligned_chunks(size_bits_logical_max, start, end):
    while start < end:
        size_bits_max = size_bits_logical_max if start == 0 else ctz(start)
        for size_bits in range(size_bits_max):
            if start + (1 << (size_bits + 1)) > end:
                break
        yield start, size_bits
        start += 1 << size_bits

def ctz(n):
    assert n > 0
    for i in itertools.count(0):
        if n & 1 << i != 0:
            return i

def mk_fill(frame_offset, length, fname, file_offset):
    return ['{} {} CDL_FrameFill_FileData "{}" {}'.format(frame_offset, length, fname, file_offset)]

###

def as_(g):
    def wrapper(f):
        def wrapped(*args, **kwargs):
            return g(*f(*args, **kwargs))
        return wrapped
    return wrapper

def as_list(f):
    def wrapped(*args, **kwargs):
        return list(f(*args, **kwargs))
    return wrapped

def groups_of(n, it):
    l = list(it)
    for i in range(0, len(l), n):
        yield l[i:i+n]
