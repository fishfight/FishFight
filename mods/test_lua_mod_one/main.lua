print("Hello from lua mod!")
require "second_file"
local entity;
return {
    init = function(world)
        print "Run init"
        local I32 = type_components.I32
        local Bool = type_components.Bool
        entity = world:spawn { I32.new(5), Bool.new(true) }
        for k,v in pairs(type_components) do
            print(k,v)
        end
    end,
    fixed_update_physics_bodies = function(world)
        -- local Query = hv.ecs.Query
        -- local I32 = type_components.I32
        -- local Bool = type_components.Bool
        -- local query = Query.new { Query.write(I32), Query.read(Bool) }
        -- world:query_one(query, entity, function(item)
            -- print("Got item:",item)
           --  -- Querying allows us to access components of our item as userdata objects through the same interface
           -- -- we defined above!
            -- print("asserting if still true")
            -- assert(item:take(Bool).value == true)
            -- local i = item:take(I32)
            -- print("asserting if still bigger than 5")
            -- assert(i.value >= 5)
            -- print("time to increment by 1")
            -- i.value = i.value + 1
            -- print("Current value:", i.value)
        -- end)
    end
}