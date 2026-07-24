GreedySplineCorridor
    Input: a spline S,|S|=n and an error corridor size e 
    Output: a spline connecting S[1],S[n] through the corridor

    B=S[1],R=<B> // S[1] is the first base point
    U=S[2] + e, L =S[2] −e // error corridor bounds

    for i=3 to n
        C=S[i]
        if BC is left of BU or right of BL
            B=S[i−1],R=R◦<B>
            U=C+e, L =C−e
        else
            U'=C+e, L'=C−e
            if BU is left of BU'
                U=U'
            if BL is right of BL'
                L=L'
    R=R◦<S[n]>
    return R


1. Take sorted keys
2. Build GreedySplineCorridor
