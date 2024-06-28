function recursive_fib(n)
  local function inner(m)
    if m < 2 then
      return m
    end
    foo = inner(m - 1)
    bar = inner(m - 2)
    res = foo + bar
    return res
  end
  res = inner(n)
  return res
end

res = recursive_fib(10)
print("Fib of 10 is " .. res)
