local min = math.min
local max = math.max

function update(storages, dt)
    for _, storage in ipairs(storages) do
        local imbalance = get_imbalance(storage.net_id)
        if imbalance ~= nil then
            storage.stored = min(storage.capacity, max(storage.stored + imbalance * dt, 0))
        end
    end
end

function on_mouse_button_down(storage, _dt)
    print(storage.stored)
end
