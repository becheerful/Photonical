function init(world, ecs, collimator, _dt)
    local max = math.max
    local min = math.min

    local ww = get_world_width(world)
    local wh = get_world_height(world)

    local sx = collimator.pos[1]
    local sy = collimator.pos[2]
    local radius = collimator.radius
    local entity_id = collimator.entity_id

    for x = max(sx - 1, 0), max(sx - radius, 0), -1 do
        local e = get_entity_at(world, x, sy)
        if e ~= 0 and e ~= entity_id then
            print(get_name(get_entity_table(ecs, e).raw_id))
            break
        end
    end

    for x = min(sx + 1, ww), min(sx + radius, ww), 1 do
        local e = get_entity_at(world, x, sy)
        if e ~= 0 and e ~= entity_id then
            print(get_name(get_entity_table(ecs, e).raw_id))
            break
        end
    end

    for y = max(sy - 1, 0), max(sy - radius, 0), -1 do
        local e = get_entity_at(world, sx, y)
        if e ~= 0 and e ~= entity_id then
            print(get_name(get_entity_table(ecs, e).raw_id))
            break
        end
    end

    for y = min(sy + 1, wh), min(sy + radius, wh), 1 do
        local e = get_entity_at(world, sx, y)
        if e ~= 0 and e ~= entity_id then
            print(get_name(get_entity_table(ecs, e).raw_id))
            break
        end
    end
end
