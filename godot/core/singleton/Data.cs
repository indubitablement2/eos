using Godot;
using System;
using System.Collections.Generic;

public static class Data
{
    // Use resource path as key.
    public static Dictionary<string, EntityData> EntityDatas;
    public static Dictionary<string, ShipData> ShipDatas;

    public static void LoadData()
    {
        EntityDatas = new Dictionary<string, EntityData>();
        ShipDatas = new Dictionary<string, ShipData>();

        Dictionary<string, string> shipDataEntityDataPaths = new Dictionary<string, string>();

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
                    GD.Print(resourcePath);

                    Resource resource = GD.Load(currentPath + fileName);
                    if (resource.HasSignal("is_entity_data"))
                    {
                        EntityData entityData = new EntityData(resource);
                        EntityDatas.Add(resourcePath, entityData);
                    }
                    else if (resource.HasSignal("is_ship_data"))
                    {
                        ShipData shipData = new ShipData(resource);
                        ShipDatas.Add(resourcePath, shipData);

                        shipDataEntityDataPaths.Add(resourcePath, (string)resource.Get("EntityDataPath"));
                    }
                }
                fileName = dirAccess.GetNext();
            }
        }

        foreach (KeyValuePair<string, string> pair in shipDataEntityDataPaths)
        {
            ShipDatas[pair.Key].EntityData = EntityDatas[pair.Value];
        }

        foreach (string path in ShipDatas.Keys)
        {
            GD.Print(path);
        }
    }
}