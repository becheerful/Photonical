function update(this, dt)
    local pos = this:get_pos()
    local block = get_block_at(pos.x + 1, pos.y)
    print(block:get_name())
end
