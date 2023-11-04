labels = {}
for i = 1, 32, 1 do
    labels[i] = property.getText(string.format("label_%d", i));
end

values = {};
tick = 0;
function onTick()
    if not input.getBool(1) then
        return;
    end
    for i = 1, 32, 1 do
        if labels[i] ~= "" and labels[i] ~= nil then
            table.insert(values, { labels[i], input.getNumber(i) });
        end
    end
    tick = tick - 1;
    if tick <= 0 then
        local query = "/push?"
        for index, value in ipairs(values) do
            if index ~= 1 then
                query = query .. "&";
            end
            query = query .. string.format("%s=%.2f", value[1], value[2]);
        end
        values = {};
        async.httpGet(8080, query);
        tick = 15;
    end
end

function httpReply(port, request_body, response_body)
    tick = 0;
end
