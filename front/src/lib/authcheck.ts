export async function authCheck() {
    try {
        const response = await fetch('/api/auth/check', {
            method: 'GET',
            credentials: 'include'
        });

        if (!response.ok) {
            return false // If the session is not valid
        }
        return true
    } catch (error) {
        console.error('An error occurred:', error);
        return false
    }
}
