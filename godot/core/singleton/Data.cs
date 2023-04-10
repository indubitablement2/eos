using Godot;
using System;
using System.Collections.Generic;

public static class Data
{
    public static Dictionary<string, EntityData> EntityDatas;

    public static void LoadData()
    {
        EntityDatas = new Dictionary<string, EntityData>();

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
                else if (fileName.EndsWith(".res"))
                {
                    Resource resource = GD.Load(currentPath + fileName);
                    if (resource.HasSignal("is_entity_data"))
                    {
                        EntityData entityData = new EntityData(resource);
                        EntityDatas.Add(entityData.Id, entityData);
                    }
                }
                fileName = dirAccess.GetNext();
            }
        }

    }
}