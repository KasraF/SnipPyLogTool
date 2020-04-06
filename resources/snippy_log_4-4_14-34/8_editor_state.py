""" 
Count Duplicates
    Return the number of characters
    that appear in the input more
    than once.
    >>> task('aabbcde') == 2
    >>> task('book') == 1
    >>> task('vscode') == 0
"""
def task(inp):
    count = {var: inp.count(var) for var in inp}
    filter = {key: count[key] for key in count if inp.count(key) > 1}
    x = range(123)
    rs = 0
    return 0

task('aabbcde')
