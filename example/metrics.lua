BASE64_CHARS = {
    [0] = 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P',
    'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b', 'c', 'd', 'e', 'f',
    'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v',
    'w', 'x', 'y', 'z', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '-', '_'
}

labels = {}
count = 0
for i = 1, 32, 1 do
    labels[i] = property.getText(string.format("label_%d", i));
    if #labels[i] > 0 then
        count = count + 1
    end
end

values = {}
tick = 0

function onTick()
    if not input.getBool(1) then
        if #values == 0 then
            return
        end
        tick = 1
    end
    for i = 1, 32, 1 do
        if labels[i] ~= "" and labels[i] ~= nil then
            values[i] = values[i] or {}
            table.insert(values[i], input.getNumber(i))
        end
    end
    tick = tick - 1
    if tick > 0 then
        return
    end

    buf = {}
    if #values < 16 then
        table.insert(buf, ('>B'):pack(0x80 + count))
    else
        table.insert(buf, ('>BI2'):pack(0xde, count))
    end
    for i, value in pairs(values) do
        local key = labels[i]
        table.insert(buf, ('>B'):pack(0xa0 + #key) .. key)
        if #value < 16 then
            table.insert(buf, ('>B'):pack(0x90 + #value))
        else
            table.insert(buf, ('>BI2'):pack(0xdc, #value))
        end
        for i, v in ipairs(value) do
            table.insert(buf, ('>Bf'):pack(0xca, v))
        end
    end
    query = "/p?" .. encode64(table.concat(buf))
    values = {}
    async.httpGet(8080, query)
    tick = 10
end

function encode64(s)
    local p = 2 - (#s - 1) % 3
    return (s .. string.rep("\0", p)):gsub("...", function(cs)
        local c1, c2, c3 = string.byte(cs, 1, 3)
        return BASE64_CHARS[c1 >> 2] ..
            BASE64_CHARS[(c1 & 0x03) << 4 | c2 >> 4] ..
            BASE64_CHARS[(c2 & 0x0f) << 2 | c3 >> 6] ..
            BASE64_CHARS[c3 & 0x3f]
    end)
end

function onDraw()
end

function httpReply(port, request_body, response_body)
    tick = 0
end
