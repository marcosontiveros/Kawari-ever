-- ScreenImageTest.lua: Demonstrates how to use the Lua API to send a ScreenImage to the client. ScreenImages are the large text banners that pop up when you unlock something major (e.g., "LAND ACQUIRED!" when you purchase a housing plot).

required_rank = GM_RANK_DEBUG
command_sender = "[screenimagetest] "

function onCommand(player, args, name)
    local usage <const> = "Displays a given ScreenImage banner. See the ScreenImage Excel sheet for possible values.\nUsage: !screenimagetest <screenimage_id>"

    if #args ~= 1 then
        printf(player, "%s", usage)
        return
    end

    local screenimage_id <const> = tonumber(args[1])

    if screenimage_id ~= nil then 
        player:show_screenimage(screenimage_id)
    else
        printf(player, "Error parsing ScreenImage id!\n"..usage)
    end
end
