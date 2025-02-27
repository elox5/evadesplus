
export type Vector2 = {
    x: number;
    y: number;
}

export type Rect = {
    x: number;
    y: number;
    w: number;
    h: number;
}

export type Portal = {
    x: number;
    y: number;
    w: number;
    h: number;
    color: string;
}

export type RenderNode = {
    x: number;
    y: number;
    radius: number;
    color: string;
    has_outline: boolean;
    is_hero: boolean;
    downed: boolean;
    player_id: bigint | null;
}