
function test_function()
    int_var = 5
    if int_var > 100 then
        print("This is inside an if block")
    elseif int_var > 1 then
        print("This is inside an elseif")
    else
        print("This is inside an else")
    end
    hello_msg = "Hello, "
    world_msg = "World!"
    print(hello_msg .. world_msg)

    print(hello_msg .. int_var)
end

test_function()
