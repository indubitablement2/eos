using Godot;
using System;

public class ShipData
{
    public EntityData EntityData;

    public String DisplayName;

    public ShipData(Resource shipDataResource)
    {
        DisplayName = (string)shipDataResource.Get("DisplayName");
    }
}