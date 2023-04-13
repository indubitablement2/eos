using Godot;
using System;
using System.Collections.Generic;

public static class Data
{
    public static Dictionary<string, ShipData> ShipDatas;

    public static void LoadData()
    {
        ShipDatas = new Dictionary<string, ShipData>();

        Stack<string> dirs = new Stack<string>();
        dirs.Push("res://");
        while (dirs.Count > 0)
        {
            string currentPath = dirs.Pop();
            DirAccess dirAccess = DirAccess.Open(currentPath);
            dirAccess.ListDirBegin();
            string fileName = dirAccess.GetNext();
            while (fileName.Length > 0)
            {
                if (dirAccess.CurrentIsDir())
                {
                    dirs.Push(currentPath + fileName + "/");
                }
                else if (fileName.EndsWith("res"))
                {
                    string resourcePath = currentPath + fileName;
                    Resource resource = GD.Load(resourcePath);
                    if (resource is ShipData)
                    {
                        ShipData shipData = (ShipData)resource;
                        shipData.FetchBaseStats();
                        ShipDatas.Add(resourcePath, shipData);
                    }
                }
                fileName = dirAccess.GetNext();
            }
        }
    }

    public static void PrintData()
    {
        GD.Print("ShipDatas:");
        foreach (KeyValuePair<string, ShipData> pair in ShipDatas)
        {
            GD.Print("  path: ", pair.Key);
            GD.Print("  name: ", pair.Value.DisplayName);
            GD.Print("  entity ship scene is valid: ", pair.Value.EntityShipScene != null);
            GD.Print("  icon is valid: ", pair.Value.Icon != null);
        }
    }
}