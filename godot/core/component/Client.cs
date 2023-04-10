using Godot;
using System;

public class Client
{
    public Int64 ClientId;

    public Client(Int64 clientId)
    {
        ClientId = clientId;
    }

    public bool IsLocal()
    {
        return ClientId == Metascape.LocalClientId;
    }
}